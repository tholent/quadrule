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

//! The emoji picker (rung 11a, docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "The emoji
//! picker"): a window with a search box (by emoji name), the nine groups, and every emoji drawn from
//! Noto Emoji. Names and groups come from the `emojis` crate (MIT or Apache-2.0, with Unicode data
//! under Unicode-3.0, whose notice ships with the packages). An emoji is a character in a text box
//! (rule Q7): the picker types it into the Text tab's field at the cursor, or, with no text box
//! picked, makes a new text box holding only it, 24 pt, in the middle of the view, as one undo step.
//! No shortcut (rung 17).

use crate::fonts::{Pictures, raster_text};
use crate::view::ViewerUi;
use egui::{Role, ScrollArea, Sense, WidgetInfo};
use emojis::{Emoji, Group};
use quadrule_model::{Extra, FontStyle, ObjectKind, TextAlign, TextBox, Uuid};
use quadrule_text::{FontRegistry, TextRequest, lay_out};

/// The point size of a new text box holding an emoji.
pub const NEW_BOX_PT: f64 = 24.0;
/// Point size an emoji is drawn at in the picker.
const CELL_PT: f64 = 14.0;
/// Width and height of a cell, in screen points.
const CELL: f32 = 38.0;

/// The groups in the order the picker lists them, with the Fluent id of each name.
pub const GROUPS: [(Group, &str); 9] = [
    (Group::SmileysAndEmotion, "app-emoji-group-smileys"),
    (Group::PeopleAndBody, "app-emoji-group-people"),
    (Group::AnimalsAndNature, "app-emoji-group-nature"),
    (Group::FoodAndDrink, "app-emoji-group-food"),
    (Group::TravelAndPlaces, "app-emoji-group-travel"),
    (Group::Activities, "app-emoji-group-activities"),
    (Group::Objects, "app-emoji-group-objects"),
    (Group::Symbols, "app-emoji-group-symbols"),
    (Group::Flags, "app-emoji-group-flags"),
];

fn emoji_box(text: &str) -> TextBox {
    TextBox {
        content: text.into(),
        family: "Noto Emoji".into(),
        weight: 400,
        style: FontStyle::Normal,
        size_pt: CELL_PT,
        line_height: 1.25,
        letter_spacing_em: 0.0,
        align: TextAlign::Left,
        extra: Extra::new(),
    }
}

/// The emoji Noto Emoji can draw as one letter each, in the `emojis` crate's order (Unicode's
/// own), without the skin tone variants (the plain one stands for them). One layout of all of
/// them; anything the font would draw as a missing box or as several pieces is left out.
pub fn drawable(reg: &mut FontRegistry) -> Vec<&'static Emoji> {
    let all: Vec<&'static Emoji> = emojis::iter()
        .filter(|e| e.skin_tone().is_none_or(|t| t == emojis::SkinTone::Default))
        .collect();
    let boxes: Vec<TextBox> = all.iter().map(|e| emoji_box(e.as_str())).collect();
    let requests: Vec<TextRequest> = boxes
        .iter()
        .enumerate()
        .map(|(i, b)| TextRequest {
            object_id: Uuid::from_u128(i as u128 + 1),
            text: b,
            width_mm: 1000.0,
        })
        .collect();
    let Ok(laid) = lay_out(reg, "emoji", &requests) else {
        return Vec::new();
    };
    let noto: Vec<bool> = laid
        .outlines
        .tables
        .iter()
        .map(|t| t.face.family == "Noto Emoji")
        .collect();
    all.into_iter()
        .zip(&laid.outlines.texts)
        .filter(|(_, t)| {
            t.glyphs.len() == 1
                && t.glyphs
                    .iter()
                    .all(|g| g.glyph() != 0 && noto.get(g.table()).copied().unwrap_or(false))
        })
        .map(|(e, _)| e)
        .collect()
}

/// The emoji picker's state.
#[derive(Default)]
pub(crate) struct EmojiPicker {
    /// The window shows.
    pub open: bool,
    search: String,
    focus_search: bool,
    group: Option<usize>,
    list: Option<Vec<&'static Emoji>>,
    pictures: Pictures,
}

impl EmojiPicker {
    /// Opens the window; the list of drawable emoji is made on first use.
    pub(crate) fn open(&mut self, fonts: &FontRegistry) {
        self.pictures.reset(fonts);
        if self.list.is_none() {
            let mut reg = fonts.sibling();
            self.list = Some(drawable(&mut reg));
        }
        self.search.clear();
        self.focus_search = true;
        self.open = true;
    }
}

impl ViewerUi {
    /// The Emoji button of the toolbar and the Text tab.
    pub(crate) fn emoji_button(&mut self, ui: &mut egui::Ui) {
        let open = self.viewer.open.is_some();
        let r = ui
            .add_enabled(open, egui::Button::new(self.t("app-emoji")))
            .on_hover_text(self.t("app-emoji-tip"));
        if r.clicked() {
            if self.emoji_picker.open {
                self.emoji_picker.open = false;
            } else {
                let fonts = &self.viewer.fonts;
                self.emoji_picker.open(fonts);
            }
        }
    }

    /// The picker window, while it is open.
    pub(crate) fn emoji_picker_window(&mut self, ctx: &egui::Context) {
        if !self.emoji_picker.open {
            return;
        }
        let ppx = f64::from(ctx.pixels_per_point()) * 96.0 / quadrule_model::MM_PER_IN;
        let mut picked: Option<&'static Emoji> = None;
        let mut open = true;
        egui::Window::new(self.t("app-emoji-picker-title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(400.0)
            .default_height(440.0)
            .show(ctx, |ui| {
                let ink = ui.visuals().text_color();
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.emoji_picker.search)
                        .hint_text(self.strings.get("app-emoji-search"))
                        .desired_width(f32::INFINITY),
                );
                r.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::SearchInput,
                        true,
                        self.strings.get("app-emoji-search"),
                    )
                });
                if std::mem::take(&mut self.emoji_picker.focus_search) {
                    r.request_focus();
                }
                let needle = self.emoji_picker.search.trim().to_lowercase();
                // Groups: one shows at a time, unless a name is being searched for.
                ui.horizontal_wrapped(|ui| {
                    for (i, (_, id)) in GROUPS.iter().enumerate() {
                        let on = self.emoji_picker.group.unwrap_or(0) == i && needle.is_empty();
                        if ui.selectable_label(on, self.strings.get(id)).clicked() {
                            self.emoji_picker.group = Some(i);
                            self.emoji_picker.search.clear();
                        }
                    }
                });
                ui.separator();
                let want_group = GROUPS[self.emoji_picker.group.unwrap_or(0)].0;
                let list = self.emoji_picker.list.take().unwrap_or_default();
                let shown: Vec<&'static Emoji> = list
                    .iter()
                    .copied()
                    .filter(|e| {
                        if needle.is_empty() {
                            e.group() == want_group
                        } else {
                            e.name().to_lowercase().contains(&needle)
                                || e.shortcodes().any(|s| s.contains(&needle))
                        }
                    })
                    .collect();
                if shown.is_empty() {
                    ui.label(self.strings.get("app-emoji-none-found"));
                }
                let cols = ((ui.available_width() / CELL).floor() as usize).max(1);
                let rows = shown.len().div_ceil(cols);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, CELL, rows, |ui, range| {
                        for row in range {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                                for e in shown.iter().skip(row * cols).take(cols) {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(CELL, CELL),
                                        Sense::click(),
                                    );
                                    resp.widget_info(|| {
                                        WidgetInfo::labeled(Role::Button, true, e.name())
                                    });
                                    if resp.hovered() {
                                        ui.painter().rect_filled(
                                            rect,
                                            3.0,
                                            ui.visuals().widgets.hovered.bg_fill,
                                        );
                                    }
                                    let key = format!("e|{}|{ppx}", e.as_str());
                                    let tb = emoji_box(e.as_str());
                                    if let Some((id, size)) =
                                        self.emoji_picker.pictures.picture(ctx, key, &tb, ppx, true)
                                    {
                                        let r = egui::Rect::from_center_size(rect.center(), size);
                                        ui.painter().image(
                                            id,
                                            r,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
                                            ink,
                                        );
                                    }
                                    if resp.on_hover_text(e.name()).clicked() {
                                        picked = Some(e);
                                    }
                                }
                            });
                        }
                    });
                self.emoji_picker.list = Some(list);
            });
        self.emoji_picker.open = open && self.emoji_picker.open;
        if let Some(e) = picked {
            self.insert_emoji(ctx, e.as_str());
        }
    }

    /// The id of the Text tab's text field on the last frame (for tests that move its cursor).
    pub fn text_field_id(&self) -> Option<egui::Id> {
        self.text_field_id
    }

    /// Puts `emoji` where the user is writing: at the Text tab field's cursor (replacing what is
    /// selected there) when a text box is picked, else in a new text box in the middle of the view.
    pub fn insert_emoji(&mut self, ctx: &egui::Context, emoji: &str) {
        let text_box = self.viewer.selection.first().and_then(|id| {
            let o = self.viewer.doc()?.object(*id)?;
            match &o.kind {
                ObjectKind::Text(t) => Some(t.content.clone()),
                _ => None,
            }
        });
        let editable = !self.viewer.editable_selection().is_empty();
        if let (Some(content), true) = (text_box, editable) {
            let len = content.chars().count();
            let (mut lo, mut hi) = (len, len);
            if let Some(id) = self.text_field_id
                && let Some(state) = egui::TextEdit::load_state(ctx, id)
                && let Some(range) = state.cursor.char_range()
            {
                let (a, b) = (range.primary.index.0, range.secondary.index.0);
                lo = a.min(b).min(len);
                hi = a.max(b).min(len);
            }
            let at = |n: usize| {
                content
                    .char_indices()
                    .nth(n)
                    .map_or(content.len(), |(i, _)| i)
            };
            let mut next = String::with_capacity(content.len() + emoji.len());
            next.push_str(&content[..at(lo)]);
            next.push_str(emoji);
            next.push_str(&content[at(hi)..]);
            let _ = self.viewer.edit_selected(None, |x| {
                if let ObjectKind::Text(t) = &mut x.kind {
                    t.content.clone_from(&next);
                }
            });
            // The cursor goes after the emoji, and the field takes it back.
            if let Some(id) = self.text_field_id
                && let Some(mut state) = egui::TextEdit::load_state(ctx, id)
            {
                let after = lo + emoji.chars().count();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(after),
                    )));
                state.store(ctx, id);
                ctx.memory_mut(|m| m.request_focus(id));
            }
        } else if let Some(centre) = self.view_centre_px {
            self.viewer.place_text(emoji, NEW_BOX_PT, centre);
        }
    }
}

/// Draws one emoji as export pixels, for tests: `None` when it has nothing to draw.
pub fn picture_of(
    reg: &mut FontRegistry,
    emoji: &str,
    px_per_mm: f64,
) -> Option<crate::fonts::Raster> {
    raster_text(reg, &emoji_box(emoji), px_per_mm)
}
