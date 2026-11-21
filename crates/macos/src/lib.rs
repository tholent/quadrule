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

//! Quadrule's few words with macOS itself (fixes batch 2, docs/bbq/2026-11-18-fixes-before-16b.md, Q4: "the macOS
//! double-click").
//!
//! Finder opens a double-clicked `.quad` by sending the program an Apple Event, not by naming the file on its command
//! line, and AppKit hands that event to the application delegate's `application:openURLs:`. winit 0.30.13 sets its
//! own delegate, `WinitApplicationDelegate`, when its event loop is made, and panics at launch if the application's
//! delegate is anything else (`ApplicationDelegate::get` in winit's `platform_impl/macos/app_state.rs`). So
//! [`install`] leaves that delegate in place and adds the one method to its class. The method only notes each file;
//! the app takes them with [`take_opened`] at the start of a frame and opens the first as if it had been dropped on
//! the window.
//!
//! On every other system this crate is empty.

#![cfg(target_os = "macos")]

use objc2::ffi;
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::sel;
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSArray, NSObjectProtocol, NSURL};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// The class of winit's application delegate (winit 0.30.13, `platform_impl/macos/app_state.rs`).
const WINIT_DELEGATE: &str = "WinitApplicationDelegate";

/// Files macOS asked the program to open, not yet taken.
static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Wakes the window when a file arrives while it sleeps (set once the window exists).
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// `application:openURLs:` as added to winit's delegate: returns nothing, takes the delegate, the selector, the
/// application and an array of URLs (AppKit, macOS 10.13).
extern "C" fn open_urls(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    urls: *mut NSArray<NSURL>,
) {
    #[allow(unsafe_code)]
    // SAFETY: AppKit passes a valid NSArray of NSURL that it keeps alive for the call, or nil.
    if let Some(urls) = unsafe { urls.as_ref() } {
        note(urls);
    }
}

/// Notes the files among `urls` (Finder sends file URLs; anything else is skipped) and wakes the window.
fn note(urls: &NSArray<NSURL>) {
    let mut paths = Vec::new();
    for url in urls.iter() {
        #[allow(unsafe_code)]
        // SAFETY: `isFileURL` and `path` only read the URL, which `urls` keeps alive for the call.
        let path = unsafe { url.isFileURL().then(|| url.path()).flatten() };
        if let Some(p) = path {
            paths.push(PathBuf::from(p.to_string()));
        }
    }
    if paths.is_empty() {
        return;
    }
    if let Ok(mut opened) = OPENED.lock() {
        opened.extend(paths);
    }
    if let Some(wake) = WAKE.get() {
        wake();
    }
}

/// Teaches winit's application delegate to hear which files Finder asks the program to open. Call it on the main
/// thread after the event loop is made (which makes winit's delegate) and before it runs, so a file double-clicked to
/// start the program is heard too. Returns false, doing nothing, off the main thread, when winit's delegate class is
/// missing or is not the application's delegate, or when the class already has the method; the program runs on
/// either way, only without the double-click.
#[allow(unsafe_code)]
pub fn install() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let Some(class) = AnyClass::get(WINIT_DELEGATE) else {
        return false;
    };
    let app = NSApplication::sharedApplication(mtm);
    // SAFETY: reading the delegate only; winit keeps it alive for as long as its event loop.
    let Some(delegate) = (unsafe { app.delegate() }) else {
        return false;
    };
    if !delegate.isMemberOfClass(class) {
        return false;
    }
    let open: extern "C" fn(*mut AnyObject, Sel, *mut AnyObject, *mut NSArray<NSURL>) = open_urls;
    // SAFETY: the function's arguments and result match the type string ("v@:@@": void; self, _cmd, two objects) and
    // the signature AppKit declares for `application:openURLs:`; `class_addMethod` copies the type string and leaves
    // an existing method alone (returning NO).
    let added = unsafe {
        ffi::class_addMethod(
            class as *const AnyClass as *mut ffi::objc_class,
            sel!(application:openURLs:).as_ptr(),
            Some(std::mem::transmute::<
                extern "C" fn(*mut AnyObject, Sel, *mut AnyObject, *mut NSArray<NSURL>),
                unsafe extern "C" fn(),
            >(open)),
            c"v@:@@".as_ptr(),
        )
    };
    if added == ffi::NO {
        return false;
    }
    // AppKit may note which methods a delegate answers when the delegate is set, so set winit's again now the class
    // has the new one (the application holds its delegate weakly; winit's event loop keeps it alive).
    app.setDelegate(None);
    app.setDelegate(Some(&delegate));
    true
}

/// Sets what wakes the window when a file arrives (the window's repaint). Only the first call counts.
pub fn set_wake(wake: impl Fn() + Send + Sync + 'static) {
    let _ = WAKE.set(Box::new(wake));
}

/// The files macOS asked the program to open since the last call, oldest first.
pub fn take_opened() -> Vec<PathBuf> {
    OPENED
        .lock()
        .map(|mut v| std::mem::take(&mut *v))
        .unwrap_or_default()
}
