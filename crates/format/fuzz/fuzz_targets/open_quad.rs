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

//! Any bytes, opened as a `.quad`: the reader must never panic, and a file it opens and may save
//! must save, reopen and save again to the very same bytes (ADR-001 item 8).

#![no_main]

use libfuzzer_sys::fuzz_target;
use quadrule_format::QuadFile;

fuzz_target!(|data: &[u8]| {
    let Ok(file) = QuadFile::from_bytes(data) else { return };
    if file.read_only.is_some() {
        return;
    }
    let Ok(bytes) = file.to_bytes("fuzz") else { return };
    let again = QuadFile::from_bytes(&bytes).expect("a file this build wrote must reopen");
    assert_eq!(again.to_bytes("fuzz").expect("and save again"), bytes, "saving is not repeatable");
});
