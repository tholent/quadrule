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

//! Quadrule's words on screen (napkin §14). Every string a person reads goes through Project
//! Fluent; English ships first (rung 2 bbq, Q5). Headless: no UI dependencies, so the linter and
//! the command-line tools use it too.
//!
//! A [`Message`] is an id and its arguments; [`Strings::format`] turns it into text in one
//! language. Code that finds something to say (the linter) builds messages; code that shows them
//! (the app, `cargo xtask lint`) formats them.

#![forbid(unsafe_code)]

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::types::{FluentNumber, FluentNumberOptions};
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

/// The English files, built into the program. One file per area of the app.
const ENGLISH: &[(&str, &str)] = &[
    ("lint.ftl", include_str!("../locales/en-US/lint.ftl")),
    ("app.ftl", include_str!("../locales/en-US/app.ftl")),
];

/// One argument to a message.
#[derive(Clone, Debug, PartialEq)]
pub enum Arg {
    /// Text shown as is, such as an object's name.
    Text(String),
    /// A number shown with exactly this many decimals, rounded as Rust's `{:.N}` rounds.
    Num(f64, u8),
    /// Another message, formatted in place.
    Msg(Box<Message>),
    /// Other messages, formatted and joined by the message `sep`.
    List {
        /// The messages to list.
        items: Vec<Message>,
        /// The id of the message that goes between two items.
        sep: &'static str,
    },
}

/// Something to say: a message id and its named arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    /// The Fluent message id, e.g. `lint-off-page`.
    pub id: &'static str,
    /// Named arguments, e.g. `("name", Arg::Text(..))`.
    pub args: Vec<(&'static str, Arg)>,
}

impl Message {
    /// A message with no arguments.
    pub fn new(id: &'static str) -> Self {
        Message {
            id,
            args: Vec::new(),
        }
    }

    /// Adds a named argument.
    pub fn arg(mut self, name: &'static str, value: Arg) -> Self {
        self.args.push((name, value));
        self
    }

    /// Adds a text argument.
    pub fn text(self, name: &'static str, value: impl Into<String>) -> Self {
        self.arg(name, Arg::Text(value.into()))
    }

    /// Adds a number argument shown with `decimals` decimals.
    pub fn num(self, name: &'static str, value: f64, decimals: u8) -> Self {
        self.arg(name, Arg::Num(value, decimals))
    }
}

/// The strings of one language.
pub struct Strings {
    bundle: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for Strings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strings")
            .field("locales", &self.bundle.locales)
            .finish()
    }
}

impl Strings {
    /// English (en-US), the language every message is written in first.
    pub fn english() -> Self {
        Self::load("en-US", ENGLISH).expect("the built-in English strings parse")
    }

    /// Builds a language from `(file name, Fluent source)` pairs. Fails on any syntax error or a
    /// message defined twice, naming the file.
    pub fn load(locale: &str, files: &[(&str, &str)]) -> Result<Self, String> {
        let lang: LanguageIdentifier = locale.parse().map_err(|e| format!("{locale}: {e:?}"))?;
        let mut bundle = FluentBundle::new_concurrent(vec![lang]);
        // Unicode direction marks around each argument matter for right-to-left languages; with
        // English only they would show as stray characters in a terminal. Revisit with the first
        // right-to-left translation (docs/STATUS-debt.md).
        bundle.set_use_isolating(false);
        for (name, src) in files {
            let res = FluentResource::try_new(src.to_string())
                .map_err(|(_, errs)| format!("{name}: {errs:?}"))?;
            bundle
                .add_resource(res)
                .map_err(|errs| format!("{name}: {errs:?}"))?;
        }
        Ok(Strings { bundle })
    }

    /// Whether the language has message `id`.
    pub fn has(&self, id: &str) -> bool {
        self.bundle.has_message(id)
    }

    /// A message with no arguments. A missing id shows as `⟨id⟩`, never as nothing.
    pub fn get(&self, id: &str) -> String {
        self.pattern(id, None).unwrap_or_else(|_| format!("⟨{id}⟩"))
    }

    /// Formats `msg`. A missing id or argument shows as `⟨id⟩`, never as nothing; tests use
    /// [`Strings::try_format`] to fail on one.
    pub fn format(&self, msg: &Message) -> String {
        self.try_format(msg)
            .unwrap_or_else(|_| format!("⟨{}⟩", msg.id))
    }

    /// Formats `msg`, or says what is missing.
    pub fn try_format(&self, msg: &Message) -> Result<String, String> {
        let mut args = FluentArgs::new();
        for (name, value) in &msg.args {
            let v: FluentValue = match value {
                Arg::Text(s) => s.clone().into(),
                Arg::Num(n, d) => number(*n, *d),
                Arg::Msg(m) => self.try_format(m)?.into(),
                Arg::List { items, sep } => {
                    let sep = self.try_format(&Message::new(sep))?;
                    items
                        .iter()
                        .map(|i| self.try_format(i))
                        .collect::<Result<Vec<_>, _>>()?
                        .join(&sep)
                        .into()
                }
            };
            args.set(*name, v);
        }
        self.pattern(msg.id, Some(&args))
    }

    fn pattern(&self, id: &str, args: Option<&FluentArgs>) -> Result<String, String> {
        let m = self
            .bundle
            .get_message(id)
            .ok_or_else(|| format!("no message `{id}`"))?;
        let p = m
            .value()
            .ok_or_else(|| format!("message `{id}` has no value"))?;
        let mut errs = Vec::new();
        let s = self.bundle.format_pattern(p, args, &mut errs);
        if errs.is_empty() {
            Ok(s.into_owned())
        } else {
            Err(format!("message `{id}`: {errs:?}"))
        }
    }

    /// Every message id in the language, sorted.
    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = ENGLISH
            .iter()
            .flat_map(|(_, src)| message_ids(src))
            .filter(|id| self.has(id))
            .collect();
        ids.sort();
        ids
    }
}

/// The message ids defined at the start of a line in a Fluent source.
fn message_ids(src: &str) -> impl Iterator<Item = String> + '_ {
    src.lines().filter_map(|l| {
        let (id, _) = l.split_once(" =")?;
        let ok = !id.is_empty()
            && id.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        ok.then(|| id.to_string())
    })
}

/// A number with exactly `decimals` decimals, rounded the way `format!("{:.N}")` rounds, so
/// moving a message into Fluent never changes its digits.
fn number(n: f64, decimals: u8) -> FluentValue<'static> {
    let d = decimals as usize;
    let rounded: f64 = format!("{n:.d$}").parse().unwrap_or(n);
    // Fluent writes "5." when asked for zero decimals, so zero means "leave as is" (the rounded
    // value is whole already).
    let digits = (d > 0).then_some(d);
    let opts = FluentNumberOptions {
        minimum_fraction_digits: digits,
        maximum_fraction_digits: digits,
        ..Default::default()
    };
    FluentValue::Number(FluentNumber::new(rounded, opts))
}
