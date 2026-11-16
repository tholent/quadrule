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

//! Typing on the canvas, with no UI in it (rung 11b, docs/bbq/2026-11-14-rung-11-text-and-fonts.md,
//! Q3): a session on one text box, with a caret and a selection, drawn exactly on the letters the
//! export draws ([`quadrule_text::edit`] lays the box out the way the export does). While it runs the
//! page shows a *preview* copy of the document with the new text (as a drag does); Ctrl+Z inside it
//! steps back through typed words; when it ends the text is one step in the document's undo
//! (ADR-010). The window feeds in keys and the pointer and draws [`Viewer::typing_view`].

use crate::viewer::Viewer;
use quadrule_layout::rotate::rotate_point;
use quadrule_layout::{TextJob, text_box_origin, text_jobs};
use quadrule_model::edit::Command;
use quadrule_model::{ObjectKind, TextBox, Uuid};
use quadrule_text::edit::{Caret, EditBuffer, Go, Op};
use quadrule_text::{BaselineGrid, TextRequest};

/// Where a typing session puts the caret to begin with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    /// All the text selected (a box just drawn).
    All,
    /// The end of the text.
    End,
    /// At a point of the page, in device pixels (a double-click).
    At(f64, f64),
}

/// What the last edit was, to group typing into words for the session's undo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Last {
    Word,
    Gap,
    Deleting,
    Other,
}

/// The numbers the layout of the box took, kept for the session.
#[derive(Clone, Debug)]
struct Inputs {
    text: TextBox,
    width_mm: f64,
    height_mm: Option<f64>,
    paragraph_mm: f64,
    grid: Option<BaselineGrid>,
}

impl Inputs {
    fn of(job: &TextJob) -> Self {
        Inputs {
            text: job.text.clone().into_owned(),
            width_mm: job.width_mm,
            height_mm: job.height_mm,
            paragraph_mm: job.paragraph_mm,
            grid: job.grid,
        }
    }

    fn request(&self) -> TextRequest<'_> {
        TextRequest {
            height_mm: self.height_mm,
            paragraph_mm: self.paragraph_mm,
            grid: self.grid,
            ..TextRequest::new(Uuid::nil(), &self.text, self.width_mm)
        }
    }
}

/// A typing session.
pub struct Typing {
    /// The text box being typed in (the source, for a copy in a repeat).
    pub id: Uuid,
    /// The device it is typed on: the one on screen when it began.
    pub profile_id: String,
    original: String,
    inputs: Inputs,
    eb: EditBuffer,
    undo: Vec<Caret>,
    redo: Vec<Caret>,
    last: Option<Last>,
    /// The text an input method is composing, drawn at the caret until it commits.
    pub preedit: Option<String>,
    /// A drag with the pointer is selecting.
    pub selecting: bool,
    shown_text: String,
}

/// Where to draw the caret and the selection, in device pixels of the device on screen.
#[derive(Clone, Debug, PartialEq)]
pub struct TypingView {
    /// The caret: its top and bottom.
    pub caret: ((f64, f64), (f64, f64)),
    /// The selected parts of lines, each a quadrilateral (turned with the box).
    pub selection: Vec<[(f64, f64); 4]>,
    /// The box, turned with it, clockwise from the top-left.
    pub outline: [(f64, f64); 4],
    /// The width of a hair-line caret in pixels.
    pub caret_width: f64,
}

/// How a point on the page maps into a text box and back (millimetres from its top-left corner).
#[derive(Clone, Copy, Debug)]
struct Xf {
    origin: (f64, f64),
    centre: (f64, f64),
    angle: f64,
    ppmm: f64,
}

impl Xf {
    fn to_px(self, x: f32, y: f32) -> (f64, f64) {
        let p = (
            self.origin.0 + f64::from(x) * self.ppmm,
            self.origin.1 + f64::from(y) * self.ppmm,
        );
        rotate_point(p, self.centre, self.angle)
    }

    fn to_mm(self, px: (f64, f64)) -> (f32, f32) {
        let p = rotate_point(px, self.centre, -self.angle);
        (
            ((p.0 - self.origin.0) / self.ppmm) as f32,
            ((p.1 - self.origin.1) / self.ppmm) as f32,
        )
    }
}

impl Viewer {
    /// Makes a long text box on the page (`--bench-type`): a long paragraph (about 430 letters) at 9 pt, 100 mm
    /// wide. One undo step. Returns its id.
    pub fn place_long_text(&mut self) -> Option<Uuid> {
        let words = "the quick brown fox jumps over the lazy dog and keeps running through the \
                     tall grass";
        let text = format!("{words} ").repeat(5);
        let (_, shown) = self.profiles()?;
        let px = shown.mm_to_px(1.0);
        let id = self.place_text(&text, 9.0, (px * 70.0, px * 80.0))?;
        let mut o = self.doc()?.object(id)?.clone();
        o.frame.w_mm = 100.0;
        o.frame.x_mm = 20.0;
        o.frame.y_mm = 30.0;
        if let ObjectKind::Text(t) = &mut o.kind {
            t.align = quadrule_model::TextAlign::Left;
        }
        self.edit(Command::SetObject(o), None).ok()?;
        Some(id)
    }

    /// Whether a typing session is on.
    pub fn is_typing(&self) -> bool {
        self.typing.is_some()
    }

    /// The text box being typed in, if any.
    pub fn typing_id(&self) -> Option<Uuid> {
        self.typing.as_ref().map(|t| t.id)
    }

    /// The text of the session now.
    pub fn typing_text(&self) -> Option<String> {
        self.typing.as_ref().map(|t| t.eb.caret().text)
    }

    /// The caret and selection of the session now.
    pub fn typing_caret(&self) -> Option<Caret> {
        self.typing.as_ref().map(|t| t.eb.caret())
    }

    /// Begins typing in the text box `id` on the device on screen, with the caret where `start`
    /// says. False when `id` is not a text box that can be edited. Ends a session already on.
    pub fn begin_typing(&mut self, id: Uuid, start: Start) -> bool {
        self.end_typing();
        let Some((primary, shown)) = self.profiles() else {
            return false;
        };
        let Some(doc) = self.doc() else {
            return false;
        };
        let Some(o) = doc.object(id) else {
            return false;
        };
        let ObjectKind::Text(t) = &o.kind else {
            return false;
        };
        // A box drawn from saved outlines (its font is missing) cannot be typed in (rung 12a, Q6).
        if o.locked
            || self.is_read_only_text(id)
            || doc.effectively_hidden(id)
            || doc.layer_of(id).is_some_and(|l| doc.layers[l].locked)
        {
            return false;
        }
        let original = t.content.clone();
        let jobs = text_jobs(doc, &primary, &shown);
        let Some(job) = jobs.iter().find(|j| j.object_id == id) else {
            return false;
        };
        let inputs = Inputs::of(job);
        let caret = match start {
            Start::All => Caret::all_selected(&original),
            Start::End | Start::At(..) => Caret::at_end(&original),
        };
        let Ok(mut eb) = EditBuffer::new(&mut self.fonts, &inputs.request(), &caret) else {
            return false;
        };
        if let Start::At(x, y) = start
            && let Some(xf) = self.typing_xf_at(id, x, y)
        {
            let (mx, my) = xf.to_mm((x, y));
            eb.apply(
                &mut self.fonts,
                Op::At {
                    x_mm: mx,
                    y_mm: my,
                    select: false,
                },
            );
        }
        // The text box stands for itself in the selection; a copy in a repeat, or text in a group,
        // by what a click would pick.
        if let Some(t) = self.target_of(id, &self.entered.clone()) {
            self.selection = vec![t];
        }
        self.typing = Some(Typing {
            id,
            profile_id: shown.id.clone(),
            shown_text: original.clone(),
            original,
            inputs,
            eb,
            undo: Vec::new(),
            redo: Vec::new(),
            last: None,
            preedit: None,
            selecting: false,
        });
        true
    }

    /// Begins typing in the text under (x, y), if there is a text box there (a double-click): the
    /// source of a repeat's copy, or text inside a group, directly. False when there is none.
    pub fn begin_typing_at(&mut self, x: f64, y: f64, slack: f64) -> bool {
        let Some(leaf) = self.pick_leaf(x, y, slack) else {
            return false;
        };
        let is_text = self
            .doc()
            .and_then(|d| d.object(leaf))
            .is_some_and(|o| matches!(o.kind, ObjectKind::Text(_)));
        is_text && self.begin_typing(leaf, Start::At(x, y))
    }

    /// The map of the text box `id` where (x, y) falls on it (a copy in a repeat included), or on
    /// its own place when (x, y) is not on any.
    fn typing_xf_at(&mut self, id: Uuid, x: f64, y: f64) -> Option<Xf> {
        let (_, shown) = self.profiles()?;
        let page = self.page()?;
        let ppmm = shown.mm_to_px(1.0);
        let entries = || page.placed.iter().filter(|p| p.id == id);
        let hit = entries().rev().find(|p| p.shape.hit(x, y, 0.0));
        let p = hit.or_else(|| entries().find(|p| !p.copy))?;
        let (origin, centre) = text_box_origin(&p.node.frame, &shown);
        Some(Xf {
            origin: (origin.x, origin.y),
            centre: (centre.x, centre.y),
            angle: p.node.angle,
            ppmm,
        })
    }

    /// The map of the text box on its own place (not a copy's).
    fn typing_xf(&mut self) -> Option<Xf> {
        let id = self.typing_id()?;
        let (_, shown) = self.profiles()?;
        let page = self.page()?;
        let p = page.placed.iter().find(|p| p.id == id && !p.copy)?;
        let (origin, centre) = text_box_origin(&p.node.frame, &shown);
        Some(Xf {
            origin: (origin.x, origin.y),
            centre: (centre.x, centre.y),
            angle: p.node.angle,
            ppmm: shown.mm_to_px(1.0),
        })
    }

    /// Where to draw the caret, the selection and the box, in device pixels; `None` with no session.
    pub fn typing_view(&mut self) -> Option<TypingView> {
        let xf = self.typing_xf()?;
        let t = self.typing.as_ref()?;
        let g = t.eb.geometry();
        let c = g.caret;
        let caret = (
            xf.to_px(c.x0 + (c.x1 - c.x0) / 2.0, c.y0),
            xf.to_px(c.x0 + (c.x1 - c.x0) / 2.0, c.y1),
        );
        let quad = |r: quadrule_text::edit::RectMm| {
            [
                xf.to_px(r.x0, r.y0),
                xf.to_px(r.x1, r.y0),
                xf.to_px(r.x1, r.y1),
                xf.to_px(r.x0, r.y1),
            ]
        };
        let selection = g.selection.iter().map(|r| quad(*r)).collect();
        // The box: its width, and its height as the page has it.
        let w = t.inputs.width_mm as f32;
        let h = (xf.centre.1 - xf.origin.1) * 2.0 / xf.ppmm;
        let outline = quad(quadrule_text::edit::RectMm {
            x0: 0.0,
            y0: 0.0,
            x1: w,
            y1: h as f32,
        });
        Some(TypingView {
            caret,
            selection,
            outline,
            caret_width: f64::from(c.x1 - c.x0) * xf.ppmm,
        })
    }

    /// Whether (x, y) is on the text box being typed in (or a copy of it in a repeat).
    pub fn typing_contains(&mut self, x: f64, y: f64) -> bool {
        let Some(id) = self.typing_id() else {
            return false;
        };
        self.page().is_some_and(|page| {
            page.placed
                .iter()
                .any(|p| p.id == id && p.shape.hit(x, y, 0.0))
        })
    }

    /// Does `op` in the session. Typing groups into words for the session's undo.
    pub fn type_op(&mut self, op: Op) {
        let Some(mut t) = self.typing.take() else {
            return;
        };
        let mutates = matches!(op, Op::Insert(_) | Op::Enter | Op::Backspace | Op::Delete);
        if mutates {
            let kind = match op {
                Op::Insert(c) if c.is_alphanumeric() => Last::Word,
                Op::Insert(c) if c.is_whitespace() => Last::Gap,
                Op::Backspace | Op::Delete => Last::Deleting,
                _ => Last::Other,
            };
            // A new word, or a new kind of edit, starts a step to go back to.
            let continues = matches!(
                (t.last, kind),
                (Some(Last::Word | Last::Gap), Last::Gap)
                    | (Some(Last::Word), Last::Word)
                    | (Some(Last::Deleting), Last::Deleting)
            );
            if !continues {
                t.undo.push(t.eb.caret());
            }
            t.redo.clear();
            t.last = Some(kind);
        } else {
            t.last = None;
        }
        t.preedit = None;
        t.eb.apply(&mut self.fonts, op);
        self.typing = Some(t);
        self.show_typed();
    }

    /// Types a string (a paste or a composed word): one step in the session's undo.
    pub fn type_str(&mut self, s: &str) {
        let Some(mut t) = self.typing.take() else {
            return;
        };
        t.undo.push(t.eb.caret());
        t.redo.clear();
        t.last = Some(Last::Other);
        t.preedit = None;
        let clean = s.replace("\r\n", "\n").replace('\r', "\n");
        for c in clean.chars() {
            let op = match c {
                '\n' => Op::Enter,
                c if c.is_control() => continue,
                c => Op::Insert(c),
            };
            t.eb.apply(&mut self.fonts, op);
        }
        self.typing = Some(t);
        self.show_typed();
    }

    /// The pointer at (x, y) device pixels: puts the caret there, or with `select` stretches the
    /// selection to it (a drag, or Shift); with `word` selects the word. False when (x, y) is not on
    /// the box.
    pub fn type_pointer(&mut self, x: f64, y: f64, select: bool, word: bool) -> bool {
        let Some(id) = self.typing_id() else {
            return false;
        };
        if !self.typing_contains(x, y) {
            return false;
        }
        let Some(xf) = self.typing_xf_at(id, x, y) else {
            return false;
        };
        let (mx, my) = xf.to_mm((x, y));
        let Some(t) = &mut self.typing else {
            return false;
        };
        t.last = None;
        let op = if word {
            Op::WordAt { x_mm: mx, y_mm: my }
        } else {
            Op::At {
                x_mm: mx,
                y_mm: my,
                select,
            }
        };
        t.eb.apply(&mut self.fonts, op);
        true
    }

    /// The pointer dragged on to (x, y), possibly outside the box: stretches the selection there.
    pub fn type_drag_to(&mut self, x: f64, y: f64) {
        let Some(xf) = self.typing_xf() else {
            return;
        };
        let (mx, my) = xf.to_mm((x, y));
        if let Some(t) = &mut self.typing {
            t.eb.apply(
                &mut self.fonts,
                Op::At {
                    x_mm: mx,
                    y_mm: my,
                    select: true,
                },
            );
        }
    }

    /// Steps back through typed words (Ctrl+Z inside the session).
    pub fn type_undo(&mut self) {
        let Some(mut t) = self.typing.take() else {
            return;
        };
        if let Some(prev) = t.undo.pop() {
            t.redo.push(t.eb.caret());
            self.restore(&mut t, &prev);
        }
        self.typing = Some(t);
        self.show_typed();
    }

    /// Steps forward again (Ctrl+Shift+Z inside the session).
    pub fn type_redo(&mut self) {
        let Some(mut t) = self.typing.take() else {
            return;
        };
        if let Some(next) = t.redo.pop() {
            t.undo.push(t.eb.caret());
            self.restore(&mut t, &next);
        }
        self.typing = Some(t);
        self.show_typed();
    }

    fn restore(&mut self, t: &mut Typing, caret: &Caret) {
        if let Ok(eb) = EditBuffer::new(&mut self.fonts, &t.inputs.request(), caret) {
            t.eb = eb;
        }
        t.last = None;
    }

    /// The selected text, for Ctrl+C.
    pub fn type_copy(&self) -> Option<String> {
        self.typing.as_ref()?.eb.caret().selected_text()
    }

    /// The selected text, for Ctrl+X, deleted from the box.
    pub fn type_cut(&mut self) -> Option<String> {
        let text = self.type_copy()?;
        self.type_op(Op::Backspace);
        Some(text)
    }

    /// Moves the caret (a convenience for tests and the window's key map).
    pub fn type_go(&mut self, to: Go, select: bool) {
        self.type_op(Op::Go { to, select });
    }

    /// Sets what an input method is composing.
    pub fn type_preedit(&mut self, text: Option<String>) {
        if let Some(t) = &mut self.typing {
            t.preedit = text.filter(|s| !s.is_empty());
        }
    }

    /// The text being composed by an input method, if any.
    pub fn typing_preedit(&self) -> Option<&str> {
        self.typing.as_ref()?.preedit.as_deref()
    }

    /// Shows the typed text on the page: a preview copy of the document, until the session ends.
    fn show_typed(&mut self) {
        let Some(t) = &self.typing else {
            return;
        };
        let text = t.eb.caret().text;
        if text == t.shown_text {
            return;
        }
        let id = t.id;
        let Some(mut doc) = self.doc().cloned() else {
            return;
        };
        if let Some(o) = doc.object_mut(id)
            && let ObjectKind::Text(tb) = &mut o.kind
        {
            tb.content.clone_from(&text);
        }
        if let Some(t) = &mut self.typing {
            t.shown_text = text;
        }
        self.show_preview(doc);
    }

    /// Ends the session: the text typed becomes one step in the document's undo (none when it
    /// is as it was).
    pub fn end_typing(&mut self) {
        let Some(t) = self.typing.take() else {
            return;
        };
        let text = t.eb.caret().text;
        self.hide_preview();
        if text == t.original {
            return;
        }
        let Some(mut o) = self.doc().and_then(|d| d.object(t.id)).cloned() else {
            return;
        };
        if let ObjectKind::Text(tb) = &mut o.kind {
            tb.content = text;
        }
        let _ = self.edit(Command::SetObject(o), None);
    }
}
