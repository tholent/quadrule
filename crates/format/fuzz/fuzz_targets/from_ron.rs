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

//! Any text, read as a profile file (rung 13a): the loader must never panic, and a profile it
//! accepts must keep to the limits and write out and read back as the very same profile.

#![no_main]

use libfuzzer_sys::fuzz_target;
use quadrule_profiles::{from_ron, from_ron_checked, limits, to_ron};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    // The shipped-profile reader: an error is fine, a panic is not.
    let _ = from_ron(text);
    let Ok(p) = from_ron_checked(text) else { return };
    assert!(limits::check(&p).is_ok(), "an accepted profile is inside the limits");
    let again = from_ron_checked(&to_ron(&p)).expect("a profile this build wrote must reread");
    assert_eq!(again, p, "writing and rereading changed the profile");
});
