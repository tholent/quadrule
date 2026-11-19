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

//! New layer and Delete layer in the layers panel (docs/bbq/2026-11-18-new-and-delete-layers.md): two buttons in the
//! panel's button row and the same two items on a layer row's right-click menu; the ask-first sheet for a layer with
//! objects; the new layer's name field, open at once. What a click does is in this file and in `Viewer::new_layer` and
//! `Viewer::delete_layer`; the commands are `InsertLayer` and `RemoveLayer`.

use crate::layer_icons::LayerIcon;
use crate::panels::Renaming;
use crate::view::ViewerUi;
use egui::{Button, Popup, PopupCloseBehavior, SetOpenCommand};
use quadrule_i18n::Message;
use quadrule_model::Document;
use quadrule_model::edit::LayerDeleteBlock;

/// What a layer row's right-click menu asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RowAct {
    /// New layer, above this layer.
    New(usize),
    /// Delete layer, this one.
    Delete(usize),
}

impl ViewerUi {
    /// The layer New layer and Delete layer work on: the layer of the first picked object, or else the picked
    /// layer row. `None` when neither is picked.
    pub fn picked_layer(&self) -> Option<usize> {
        let doc = self.viewer.doc()?;
        self.viewer
            .selection
            .iter()
            .find_map(|id| doc.layer_of(*id))
            .or(self.selected_layer.filter(|l| *l < doc.layers.len()))
    }

    /// The layer a Delete layer sheet is asking about, and its name; `None` when no sheet shows.
    pub fn layer_delete_ask(&self) -> Option<(usize, &str)> {
        self.delete_layer_ask
            .as_ref()
            .map(|(i, n)| (*i, n.as_str()))
    }

    /// The layer whose name field is open, if one is.
    pub fn renaming_layer(&self) -> Option<usize> {
        match &self.renaming {
            Some(Renaming::Layer(i, _)) => Some(*i),
            _ => None,
        }
    }

    /// "Layer N" with the lowest N no layer already has.
    pub(crate) fn next_layer_name(&self, doc: &Document) -> String {
        (1..)
            .map(|n| {
                self.strings
                    .format(&Message::new("app-layer-new-name").num("number", n as f64, 0))
            })
            .find(|name| !doc.layers.iter().any(|l| l.name == *name))
            .expect("some number is free")
    }

    /// New layer: an empty layer just above layer `above` (on top when `None`), picked, with its name field open
    /// and its text selected to type over. One undo step for the layer; a name kept is a second.
    pub fn new_layer(&mut self, above: Option<usize>) {
        let Some(name) = self.viewer.doc().map(|d| self.next_layer_name(d)) else {
            return;
        };
        if let Ok(Some(index)) = self.viewer.new_layer(above, &name) {
            self.selected_layer = Some(index);
            self.viewer.selection.clear();
            self.renaming = Some(Renaming::Layer(index, name));
            self.rename_select_all = true;
        }
    }

    /// Whether Delete layer works on `layer`, and if not, the message that says why.
    pub(crate) fn delete_layer_state(&self, layer: Option<usize>) -> Result<usize, &'static str> {
        let (Some(doc), Some(li)) = (self.viewer.doc(), layer) else {
            return Err("app-layer-delete-none");
        };
        match doc.layer_delete_block(li) {
            None => Ok(li),
            Some(LayerDeleteBlock::Last) => Err("app-layer-delete-last"),
            Some(LayerDeleteBlock::Locked) => Err("app-layer-delete-locked"),
            Some(LayerDeleteBlock::HoldsLocked) => Err("app-layer-delete-holds-locked"),
        }
    }

    /// Delete layer: an empty layer goes at once; one with objects asks first.
    pub fn delete_layer(&mut self, layer: usize) {
        if self.delete_layer_state(Some(layer)).is_err() {
            return;
        }
        let Some(doc) = self.viewer.doc() else {
            return;
        };
        if doc.layer_object_count(layer) == 0 {
            self.delete_layer_now(layer);
        } else {
            self.delete_layer_ask = Some((layer, doc.layers[layer].name.clone()));
        }
    }

    fn delete_layer_now(&mut self, layer: usize) {
        if self.viewer.delete_layer(layer).is_ok() {
            self.selected_layer = None;
            self.viewer.selection.clear();
            self.renaming = None;
        }
    }

    /// The two buttons of the layers panel's icon row: New layer and Delete layer.
    pub(crate) fn layer_buttons(&mut self, ui: &mut egui::Ui) {
        let picked = self.picked_layer();
        let has_doc = self.viewer.doc().is_some();
        let hint = Some(self.t("app-layer-new-hint"));
        if self.layer_icon_button(ui, LayerIcon::NewLayer, has_doc, hint) {
            self.new_layer(picked);
        }
        let state = self.delete_layer_state(picked);
        let why = state.err().map(|why| self.t(why));
        if self.layer_icon_button(ui, LayerIcon::DeleteLayer, state.is_ok(), why)
            && let Ok(li) = state
        {
            self.delete_layer(li);
        }
    }

    /// The right-click menu of layer `li`'s row: New layer and Delete layer. A right-click also picks the row, so
    /// the buttons and the menu work on the same layer.
    pub(crate) fn layer_row_menu(&mut self, row: &egui::Response, li: usize) -> Option<RowAct> {
        if row.container_secondary_clicked() {
            self.selected_layer = Some(li);
            self.viewer.selection.clear();
        }
        // `--look layers-menu` opens it once on the picked row, where the pointer is not.
        let look = self.look_layer_menu && self.selected_layer == Some(li);
        let open = if look {
            // Held open every frame: egui throws a frame away while it sizes a new popup, and a one-time open
            // would be thrown with it. (A look is for a screenshot; the menu does not close.)
            Some(SetOpenCommand::Bool(true))
        } else if row.container_secondary_clicked() {
            Some(SetOpenCommand::Bool(true))
        } else if row.container_clicked() {
            Some(SetOpenCommand::Bool(false))
        } else {
            None
        };
        let popup = Popup::menu(row)
            .open_memory(open)
            .close_behavior(PopupCloseBehavior::CloseOnClick);
        let popup = if look {
            popup.at_position(row.rect.left_center() + egui::vec2(70.0, 8.0))
        } else {
            popup.at_pointer_fixed()
        };
        let new_text = self.t("app-layer-new");
        let delete_text = self.t("app-layer-delete");
        let state = self.delete_layer_state(Some(li));
        let why = state.err().map(|id| self.t(id));
        let mut act = None;
        popup.show(|ui| {
            if ui.button(new_text).clicked() {
                act = Some(RowAct::New(li));
            }
            let delete = ui.add_enabled(state.is_ok(), Button::new(delete_text));
            let delete = match &why {
                Some(w) => delete.on_disabled_hover_text(w.clone()),
                None => delete,
            };
            if delete.clicked() {
                act = Some(RowAct::Delete(li));
            }
        });
        act
    }

    /// Does what a row's menu asked.
    pub(crate) fn do_row_act(&mut self, act: RowAct) {
        match act {
            RowAct::New(li) => self.new_layer(Some(li)),
            RowAct::Delete(li) => self.delete_layer(li),
        }
    }

    /// Drops what points at a layer that is no longer there (Undo took it away, or a file opened).
    pub(crate) fn drop_stale_layer_state(&mut self) {
        let n = self.viewer.doc().map_or(0, |d| d.layers.len());
        if self.selected_layer.is_some_and(|l| l >= n) {
            self.selected_layer = None;
        }
        if self.renaming_layer().is_some_and(|l| l >= n) {
            self.renaming = None;
        }
        if self.delete_layer_ask.as_ref().is_some_and(|(l, name)| {
            self.viewer
                .doc()
                .is_none_or(|d| d.layers.get(*l).is_none_or(|x| x.name != *name))
        }) {
            self.delete_layer_ask = None;
        }
    }

    /// The sheet that asks before a layer with objects goes: Delete or Cancel (Esc is Cancel).
    pub(crate) fn delete_layer_sheet(&mut self, ctx: &egui::Context) {
        self.drop_stale_layer_state();
        let Some((layer, name)) = self.delete_layer_ask.clone() else {
            return;
        };
        let count = self.viewer.doc().map_or(0, |d| d.layer_object_count(layer));
        let ask = self.strings.format(
            &Message::new("app-layer-delete-ask")
                .text("name", name)
                .num("count", count as f64, 0),
        );
        let mut answer = None;
        let modal = egui::Modal::new(egui::Id::unique("delete-layer")).show(ctx, |ui| {
            ui.set_max_width(380.0);
            ui.heading(ask);
            ui.add_space(6.0);
            ui.label(self.t("app-layer-delete-undo"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-layer-delete-yes")).clicked() {
                    answer = Some(true);
                }
                if ui.button(self.t("app-cancel")).clicked() {
                    answer = Some(false);
                }
            });
        });
        if answer.is_none() && modal.should_close() {
            answer = Some(false);
        }
        match answer {
            Some(true) => {
                self.delete_layer_ask = None;
                self.delete_layer_now(layer);
            }
            Some(false) => self.delete_layer_ask = None,
            None => {}
        }
    }

    /// `--look layers|layers-new|layers-ask|layers-menu`, for a screenshot: the sample page with a third layer,
    /// "Labels", on top, and the middle one ("Header") picked. `layers-new` then presses New layer; `layers-ask` Delete
    /// layer on the layer with objects (its sheet asks); `layers-menu` opens that row's right-click menu.
    pub fn look_layers(&mut self, look: &str) {
        if self.viewer.doc().is_none() {
            return;
        }
        let _ = self.viewer.new_layer(None, "Labels");
        self.selected_layer = Some(1);
        self.viewer.selection.clear();
        match look {
            "layers-new" => self.new_layer(Some(1)),
            "layers-ask" => self.delete_layer(1),
            "layers-menu" => self.look_layer_menu = true,
            _ => {}
        }
    }
}
