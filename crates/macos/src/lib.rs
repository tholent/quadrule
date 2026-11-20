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
//! line, and AppKit hands that event to the application delegate's `application:openURLs:`. winit 0.30 installs no
//! delegate of its own and says to install one after its event loop is made and before it runs (winit's
//! `platform::macos` notes); [`install`] does that. The delegate only notes each file; the app takes them with
//! [`take_opened`] at the start of a frame and opens the first as if it had been dropped on the window.
//!
//! On every other system this crate is empty.

#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, DeclaredClass, declare_class, msg_send_id, mutability};
use objc2_app_kit::{NSApplication, NSApplicationDelegate};
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSObjectProtocol, NSURL};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Files macOS asked the program to open, not yet taken.
static OPENED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Wakes the window when a file arrives while it sleeps (set once the window exists).
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

#[allow(unsafe_code)]
mod delegate {
    use super::*;

    declare_class!(
        pub(super) struct Delegate;

        // SAFETY: NSObject has no subclassing rules to keep; the delegate is made and used on the main thread only
        // (`MainThreadOnly`), as AppKit calls it.
        unsafe impl ClassType for Delegate {
            type Super = NSObject;
            type Mutability = mutability::MainThreadOnly;
            const NAME: &'static str = "QuadruleAppDelegate";
        }

        impl DeclaredClass for Delegate {}

        // SAFETY: NSObjectProtocol asks nothing NSObject does not already do.
        unsafe impl NSObjectProtocol for Delegate {}

        // SAFETY: the one method has the signature AppKit declares for `application:openURLs:` (macOS 10.13).
        unsafe impl NSApplicationDelegate for Delegate {
            #[method(application:openURLs:)]
            fn application_open_urls(&self, _app: &NSApplication, urls: &NSArray<NSURL>) {
                super::note(urls);
            }
        }
    );

    impl Delegate {
        pub(super) fn new(mtm: MainThreadMarker) -> Retained<Self> {
            // SAFETY: `init` is NSObject's own initialiser, sent once to a freshly allocated object with no ivars.
            unsafe { msg_send_id![super(mtm.alloc().set_ivars(())), init] }
        }
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

/// Installs the delegate that hears which files Finder asks the program to open. Call it on the main thread after
/// the event loop is made and before it runs, so a file double-clicked to start the program is heard too. Returns
/// false off the main thread, where it does nothing.
pub fn install() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let delegate = delegate::Delegate::new(mtm);
    let app = NSApplication::sharedApplication(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // The application keeps only a weak reference to its delegate; this one lives as long as the program.
    std::mem::forget(delegate);
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
