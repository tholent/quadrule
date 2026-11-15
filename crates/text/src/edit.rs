// Copyright 2026 Chris Wells
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Typing in a text box (rung 11b, docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Q3): our own
//! editor on cosmic-text's `Editor` and `Buffer`, over the very layout the export uses
//! ([`crate::lay_out`]'s `paragraph` and `arrange`), so a caret or a selection drawn from here sits
//! exactly on the drawn letters. Everything is in millimetres from the box's top-left corner; the
//! window turns it into pixels (and turns it with a turned box).
//!
//! The text is plain: one style per box, paragraphs split at `\n`. Positions are cosmic-text's
//! cursors in plain form ([`Pos`]: paragraph and byte offset in it).

use crate::{
    Arranged, BaselineGrid, LineSlot, Prepared, RunBox, TextError, TextRequest, arrange_buffer,
    paragraph, prepare,
};
use cosmic_text::{Action, Cursor, Edit, Editor, Motion, Selection};
use quadrule_model::{TextBox, Uuid};

/// A place in the text: the paragraph and the byte offset in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    /// The paragraph, from 0.
    pub line: usize,
    /// The byte offset in the paragraph, on a character boundary.
    pub index: usize,
}

impl From<Cursor> for Pos {
    fn from(c: Cursor) -> Self {
        Pos {
            line: c.line,
            index: c.index,
        }
    }
}

impl From<Pos> for Cursor {
    fn from(p: Pos) -> Self {
        Cursor::new(p.line, p.index)
    }
}

/// The text being typed and where the caret and the selection are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Caret {
    /// The text.
    pub text: String,
    /// Where the caret is.
    pub cursor: Pos,
    /// The other end of the selection, when there is one.
    pub anchor: Option<Pos>,
}

impl Caret {
    /// The caret at the end of `text`, nothing selected.
    pub fn at_end(text: &str) -> Self {
        Caret {
            text: text.to_string(),
            cursor: end_of(text),
            anchor: None,
        }
    }

    /// All of `text` selected (the caret at its end).
    pub fn all_selected(text: &str) -> Self {
        Caret {
            text: text.to_string(),
            cursor: end_of(text),
            anchor: Some(Pos { line: 0, index: 0 }),
        }
    }

    /// The selection from its first end to its last, or `None` when nothing is selected.
    pub fn selection(&self) -> Option<(Pos, Pos)> {
        let a = self.anchor?;
        (a != self.cursor).then(|| (a.min(self.cursor), a.max(self.cursor)))
    }

    /// The selected text, if any.
    pub fn selected_text(&self) -> Option<String> {
        let (a, b) = self.selection()?;
        let (from, to) = (offset(&self.text, a), offset(&self.text, b));
        self.text.get(from..to).map(String::from)
    }
}

/// The end of `text` as a position.
pub fn end_of(text: &str) -> Pos {
    let line = text.split('\n').count() - 1;
    let index = text.rsplit('\n').next().map_or(0, str::len);
    Pos { line, index }
}

/// The byte offset in `text` of position `p`.
pub fn offset(text: &str, p: Pos) -> usize {
    let mut at = 0;
    for (i, line) in text.split('\n').enumerate() {
        if i == p.line {
            return at + p.index.min(line.len());
        }
        at += line.len() + 1;
    }
    text.len()
}

/// Where the caret goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Go {
    /// One character left.
    Left,
    /// One character right.
    Right,
    /// A line up.
    Up,
    /// A line down.
    Down,
    /// The start of the line.
    Home,
    /// The end of the line.
    End,
    /// A word left.
    WordLeft,
    /// A word right.
    WordRight,
    /// The start of the text.
    Start,
    /// The end of the text.
    Finish,
}

/// One thing the typist does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    /// Moves the caret; with `select` the selection stretches to the new place.
    Go {
        /// Where.
        to: Go,
        /// Whether Shift is held.
        select: bool,
    },
    /// Types a character (replacing a selection).
    Insert(char),
    /// A new paragraph (replacing a selection).
    Enter,
    /// Deletes the selection, or the character before the caret.
    Backspace,
    /// Deletes the selection, or the character after the caret.
    Delete,
    /// Selects all of the text.
    SelectAll,
    /// Puts the caret where a point of the box is (millimetres from its top-left corner); with
    /// `select` the other end of the selection stays.
    At {
        /// Across.
        x_mm: f32,
        /// Down.
        y_mm: f32,
        /// Whether to stretch the selection (Shift, or a drag).
        select: bool,
    },
    /// Selects the word at a point of the box (a double-click).
    WordAt {
        /// Across.
        x_mm: f32,
        /// Down.
        y_mm: f32,
    },
}

/// A rectangle in millimetres from the box's top-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectMm {
    /// Left.
    pub x0: f32,
    /// Top.
    pub y0: f32,
    /// Right.
    pub x1: f32,
    /// Bottom.
    pub y1: f32,
}

/// What to draw for the caret and the selection.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    /// The caret: a thin box from above the letters to below them, at the caret's place.
    pub caret: RectMm,
    /// The selected parts of lines, one rectangle each.
    pub selection: Vec<RectMm>,
    /// Where the line the caret is on has its baseline, mm from the box's top.
    pub baseline_mm: f32,
}

/// The text of one box being edited, laid out as the export lays it out, with cosmic-text's editor
/// over it.
pub struct EditBuffer {
    ed: Editor<'static>,
    /// The box as it was laid out: its style (the content is the buffer's), and the numbers the
    /// layout took.
    tb: TextBox,
    width_mm: f64,
    height_mm: Option<f64>,
    paragraph_mm: f64,
    grid: Option<BaselineGrid>,
    prep: Prepared,
    /// Where the caret was asked to go up and down: the column it keeps through short lines.
    want_x: Option<f32>,
}

impl EditBuffer {
    /// The box in `req` with `caret.text` in it, ready to edit from `caret`.
    pub fn new(
        reg: &mut crate::FontRegistry,
        req: &TextRequest,
        caret: &Caret,
    ) -> Result<EditBuffer, TextError> {
        let mut tb = req.text.clone();
        tb.content.clone_from(&caret.text);
        let resolved = reg
            .resolve(&tb.family, tb.weight, tb.style, tb.size_pt)
            .ok_or_else(|| TextError::MissingFamily(tb.family.clone()))?;
        let own = TextRequest { text: &tb, ..*req };
        let prep = prepare(reg, &own, resolved)?;
        let mut buf = paragraph(
            &mut reg.fs,
            &tb,
            &prep.resolved,
            prep.size_mm,
            prep.line_mm,
            req.width_mm as f32,
        );
        buf.shape_until_scroll(&mut reg.fs, false);
        let mut ed = Editor::new(buf);
        ed.set_cursor(caret.cursor.into());
        ed.set_selection(match caret.anchor {
            Some(a) => Selection::Normal(a.into()),
            None => Selection::None,
        });
        Ok(EditBuffer {
            ed,
            tb,
            width_mm: req.width_mm,
            height_mm: req.height_mm,
            paragraph_mm: req.paragraph_mm,
            grid: req.grid,
            prep,
            want_x: None,
        })
    }

    /// The text, caret and selection now.
    pub fn caret(&self) -> Caret {
        let text = self.ed.with_buffer(|b| {
            b.lines
                .iter()
                .map(|l| l.text())
                .collect::<Vec<_>>()
                .join("\n")
        });
        let anchor = match self.ed.selection() {
            Selection::None => None,
            Selection::Normal(c) | Selection::Line(c) | Selection::Word(c) => Some(c.into()),
        };
        Caret {
            text,
            cursor: self.ed.cursor().into(),
            anchor,
        }
    }

    fn request(&self) -> TextRequest<'_> {
        TextRequest {
            object_id: Uuid::nil(),
            text: &self.tb,
            width_mm: self.width_mm,
            height_mm: self.height_mm,
            paragraph_mm: self.paragraph_mm,
            grid: self.grid,
        }
    }

    /// The runs of the buffer and where the box puts them.
    fn arranged(&self) -> (Vec<RunBox>, Arranged) {
        let req = self.request();
        self.ed.with_buffer(|b| arrange_buffer(b, &req, &self.prep))
    }

    /// Does `op`, then shapes what it changed.
    pub fn apply(&mut self, reg: &mut crate::FontRegistry, op: Op) {
        let fs = &mut reg.fs;
        let keep_x = matches!(
            op,
            Op::Go {
                to: Go::Up | Go::Down,
                ..
            }
        );
        match op {
            Op::Go { to, select } => {
                if select {
                    if matches!(self.ed.selection(), Selection::None) {
                        self.ed.set_selection(Selection::Normal(self.ed.cursor()));
                    }
                } else if let Some((a, b)) = self.caret().selection() {
                    // A plain arrow with a selection first goes to its edge, then moves on from
                    // there only for the up and down keys.
                    let edge = match to {
                        Go::Left | Go::Home | Go::WordLeft | Go::Start | Go::Up => a,
                        _ => b,
                    };
                    self.ed.set_selection(Selection::None);
                    if matches!(to, Go::Left | Go::Right) {
                        self.ed.set_cursor(edge.into());
                        self.ed.with_buffer_mut(|b| b.set_redraw(true));
                        return;
                    }
                    self.ed.set_cursor(edge.into());
                } else {
                    self.ed.set_selection(Selection::None);
                }
                match to {
                    Go::Up | Go::Down => self.go_vertical(to == Go::Up),
                    other => {
                        let motion = match other {
                            Go::Left => Motion::Left,
                            Go::Right => Motion::Right,
                            Go::Home => Motion::Home,
                            Go::End => Motion::End,
                            Go::WordLeft => Motion::LeftWord,
                            Go::WordRight => Motion::RightWord,
                            Go::Start => Motion::BufferStart,
                            _ => Motion::BufferEnd,
                        };
                        self.ed.action(fs, Action::Motion(motion));
                    }
                }
            }
            Op::Insert(c) => self.ed.action(fs, Action::Insert(c)),
            Op::Enter => self.ed.action(fs, Action::Enter),
            Op::Backspace => self.ed.action(fs, Action::Backspace),
            Op::Delete => self.ed.action(fs, Action::Delete),
            Op::SelectAll => {
                let end = end_of(&self.caret().text);
                self.ed.set_selection(Selection::Normal(Cursor::new(0, 0)));
                self.ed.set_cursor(end.into());
            }
            Op::At { x_mm, y_mm, select } => {
                let at = self.hit(x_mm, y_mm);
                if select {
                    if matches!(self.ed.selection(), Selection::None) {
                        self.ed.set_selection(Selection::Normal(self.ed.cursor()));
                    }
                } else {
                    self.ed.set_selection(Selection::None);
                }
                self.ed.set_cursor(at.into());
            }
            Op::WordAt { x_mm, y_mm } => {
                let at = self.hit(x_mm, y_mm);
                self.ed.set_selection(Selection::None);
                self.ed.set_cursor(at.into());
                self.ed.action(fs, Action::Motion(Motion::LeftWord));
                let start = self.ed.cursor();
                self.ed.set_cursor(at.into());
                self.ed.action(fs, Action::Motion(Motion::RightWord));
                let end = self.ed.cursor();
                // On a space, or at the very end, the word is whatever the motions found.
                self.ed.set_selection(Selection::Normal(start));
                self.ed.set_cursor(end);
            }
        }
        if !keep_x {
            self.want_x = None;
        }
        self.ed.with_buffer_mut(|b| b.shape_until_scroll(fs, false));
    }

    /// Moves the caret a line up or down, keeping the column it started from through short lines.
    fn go_vertical(&mut self, up: bool) {
        let (runs, _) = self.arranged();
        let cursor = self.ed.cursor();
        let Some(now) = self.run_of(cursor) else {
            return;
        };
        let x = match self.want_x {
            Some(x) => x,
            None => self
                .ed
                .with_buffer(|b| {
                    b.layout_runs()
                        .nth(now)
                        .and_then(|r| r.cursor_position(&cursor))
                })
                .unwrap_or(0.0),
        };
        self.want_x = Some(x);
        let next = if up {
            now.checked_sub(1)
        } else {
            (now + 1 < runs.len()).then_some(now + 1)
        };
        let target = match next {
            Some(j) => self.cursor_in_run(j, x),
            // Up from the first line goes to the start, down from the last to the end.
            None => {
                let end = end_of(&self.caret().text);
                if up { Pos::default() } else { end }.into()
            }
        };
        self.ed.set_cursor(target);
    }

    /// The index of the run the cursor is on.
    fn run_of(&self, c: Cursor) -> Option<usize> {
        self.ed.with_buffer(|b| {
            b.layout_runs()
                .position(|r| r.cursor_position(&c).is_some())
        })
    }

    /// The cursor in run `j` at column `x`.
    fn cursor_in_run(&self, j: usize, x: f32) -> Cursor {
        self.ed.with_buffer(|b| {
            let run = b.layout_runs().nth(j);
            match run {
                Some(r) => b
                    .hit(x, r.line_top + r.line_height / 2.0)
                    .unwrap_or_else(|| Cursor::new(r.line_i, 0)),
                None => Cursor::new(0, 0),
            }
        })
    }

    /// The cursor nearest to a point of the box (mm from its top-left corner).
    pub fn hit(&self, x_mm: f32, y_mm: f32) -> Pos {
        let (runs, arranged) = self.arranged();
        if runs.is_empty() {
            return Pos::default();
        }
        // The line whose slot holds the point, or the nearest: above the first, below the last,
        // and a paragraph's space goes to the line after it.
        let j = arranged
            .lines
            .iter()
            .position(|l| y_mm < l.bottom)
            .unwrap_or(runs.len() - 1);
        self.cursor_in_run(j, x_mm).into()
    }

    /// Where to draw the caret and the selection.
    pub fn geometry(&self) -> Geometry {
        let (_, arranged) = self.arranged();
        let caret = self.caret();
        let cursor: Cursor = caret.cursor.into();
        let size = self.prep.size_mm;
        let fallback = LineSlot {
            shift: 0.0,
            top: 0.0,
            bottom: size,
            visible: true,
        };
        let j = self.run_of(cursor).unwrap_or(0);
        let slot = arranged.lines.get(j).copied().unwrap_or(fallback);
        let (x, base) = self.ed.with_buffer(|b| {
            let run = b.layout_runs().nth(j);
            (
                run.as_ref()
                    .and_then(|r| r.cursor_position(&cursor))
                    .unwrap_or(0.0),
                run.map_or(size, |r| r.line_y) + slot.shift,
            )
        });
        // As wide as a hair: a tenth of the stroke of a letter.
        let hair = size * 0.045;
        let rect = RectMm {
            x0: x - hair / 2.0,
            y0: base - size * 0.82,
            x1: x + hair / 2.0,
            y1: base + size * 0.22,
        };
        let mut selection = Vec::new();
        if let Some((a, b)) = caret.selection() {
            let (ca, cb): (Cursor, Cursor) = (a.into(), b.into());
            self.ed.with_buffer(|buf| {
                for (j, run) in buf.layout_runs().enumerate() {
                    let Some(slot) = arranged.lines.get(j) else {
                        continue;
                    };
                    for (x, w) in run.highlight(ca, cb) {
                        selection.push(RectMm {
                            x0: x,
                            y0: slot.top,
                            x1: x + w,
                            y1: slot.bottom,
                        });
                    }
                }
            });
        }
        Geometry {
            caret: rect,
            selection,
            baseline_mm: base,
        }
    }
}
