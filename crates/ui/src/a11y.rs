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

//! Names for screen readers (rung 17 bbq, Q5): the words a control shows are its name unless two controls in one
//! window would then share it; then the name is the shown words with what the control is about after them
//! (`Show in Basic: Title`). A name always starts with the shown words, so voice control that says what it sees still
//! works. `crates/ui/tests/screen_readers.rs` fails on an unnamed control and on a shared name.

use egui::{Response, Role, WidgetInfo};

/// Names a button `name` for screen readers.
pub(crate) fn name_button(r: &Response, enabled: bool, name: &str) {
    r.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, name));
}

/// Names a tick box `name`, keeping its state.
pub(crate) fn name_check(r: &Response, enabled: bool, checked: bool, name: &str) {
    r.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, checked, name));
}

/// Names a number field `name`, keeping its value.
pub(crate) fn name_spin(r: &Response, enabled: bool, value: f64, name: &str) {
    r.widget_info(|| WidgetInfo {
        label: Some(name.to_string()),
        value: Some(value),
        enabled,
        ..WidgetInfo::new(Role::SpinButton)
    });
}
