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

//! The registry the app looks profiles up in (rung 13a bbq, Defaulted "One registry"). One per thread:
//! the window and everything that draws run on one thread, and a test gets a registry of its own, so
//! tests never see each other's profiles or the user's config folder. A thread starts with the
//! built-ins only ([`Registry::builtin_only`]); the app puts the loaded registry in with [`set`].

use crate::{Orientation, Profile, ProfileError, Registry};
use std::cell::RefCell;

thread_local! {
    static ACTIVE: RefCell<Registry> = RefCell::new(Registry::builtin_only());
}

/// Makes `r` this thread's registry.
pub fn set(r: Registry) {
    ACTIVE.with(|a| *a.borrow_mut() = r);
}

/// Reads this thread's registry.
pub fn with<T>(f: impl FnOnce(&Registry) -> T) -> T {
    ACTIVE.with(|a| f(&a.borrow()))
}

/// Changes this thread's registry. Do not call anything here that looks a profile up.
pub fn with_mut<T>(f: impl FnOnce(&mut Registry) -> T) -> T {
    ACTIVE.with(|a| f(&mut a.borrow_mut()))
}

/// The profile `id` stands for, as the device's own portrait page.
pub fn lookup(id: &str) -> Result<Profile, ProfileError> {
    with(|r| r.get(id).cloned()).ok_or_else(|| ProfileError::Unknown(id.into()))
}

/// The profile `id` stands for, as a design held `orientation` sees it.
pub fn lookup_for(id: &str, orientation: Orientation) -> Result<Profile, ProfileError> {
    Ok(lookup(id)?.oriented(orientation))
}

/// The ids of the devices a page can target here: the built-ins, then the user's own.
pub fn device_ids() -> Vec<String> {
    with(|r| r.device_ids())
}

/// Whether built-in `id` shows with a local override (the **Edited** mark).
pub fn is_edited(id: &str) -> bool {
    with(|r| r.is_edited(id))
}

/// The name shown for `id`, if it is known.
pub fn display_name(id: &str) -> Option<String> {
    with(|r| r.get(id).map(|p| p.display_name.clone()))
}
