---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"ratify 025\")"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: process
built: landed
proof: "`.github/workflows/release.yml` (job \"Homebrew tap\"), and tholent/homebrew-tap run 45370416264: style, audit and install of quadrule 0.16.6 passed"
supersedes: nothing
amends: nothing
---
# ADR-025 — Quadrule installs on macOS with Homebrew, as a cask in tholent's shared tap that follows each release

## Decision

1. **A cask, Mac only.** `brew install --cask tholent/tap/quadrule` puts the release's universal `.dmg`'s
   `Quadrule.app` in Applications. There is no Homebrew formula (command-line build) and nothing for Homebrew on
   Linux.
2. **tholent's shared tap, `github.com/tholent/homebrew-tap`.** One tap holds every cask and formula tholent makes,
   quadrule's first. The cask lives at `Casks/quadrule.rb` there, not in Homebrew's main collection and not in this
   repository.
3. **The release moves it.** The cask's source is `packaging/homebrew/quadrule.rb` in this repository. After a
   release is published, `.github/workflows/release.yml` (job "Homebrew tap") fills in the version and the `.dmg`'s
   sha256 and pushes it to the tap, with a deploy key that can write to the tap only (secret
   `HOMEBREW_TAP_DEPLOY_KEY`), and touches no other file in the tap. A `-test` tag never moves it.
4. **Gatekeeper is left alone.** The cask does not clear macOS's quarantine mark. After installing it prints the same
   "Open Anyway" steps as the README's Download section (spec §15.4).

## Context

chris asked, 2026-11-23: "let's setup this repo to allow installation from homebrew". Each release already attaches
an unsigned universal `.dmg` (`.github/workflows/release.yml`; `gh release view v0.16.5` lists
`quadrule-0.16.5-macos-universal.dmg`). The spec ships v1.0 "as a GitHub release only" (`napkins/quadrule-spec.md`
§15.4, amendment of 2026-11-10), so Homebrew is a new channel beside it, not instead of it.

Homebrew's main cask collection does not take an unsigned app: Homebrew 5.0.0 deprecated casks that fail
Gatekeeper and said they would be disabled in September 2026, and dropped the `--no-quarantine` flag
(brew.sh/2025/11/12/homebrew-5.0.0; brew.sh/2026/06/11/homebrew-6.0.0). A tap of our own is the only home until
the app is signed. chris wants one tap for all his casks and formulas (2026-11-23: "I would like there to be a
tholent/homebrew repository that holds all my casks/formulas"); Homebrew finds `tholent/<name>` at
`github.com/tholent/homebrew-<name>`, so a repository named exactly `tholent/homebrew` would need its URL typed when
tapping, and chris chose `tholent/homebrew-tap`.

Drivers:
- Install and update with one command on a Mac, getting the same app the release ships.
- Nothing to remember at release time: the tag push stays the whole release (DEVELOPMENT.md, "Releases and
  packages").
- Do not weaken a macOS safety check on the user's behalf.

## Alternatives

### A cask for the Mac app (chosen: chris's ruling)

- Good: the real `Quadrule.app`, so the Dock icon, the menu bar and Finder's double-click on a `.quad` work; the
  same file every release already builds.
- Bad: Mac only; Gatekeeper warns on first open until the app is signed.

### A cask plus a Linux formula

- Good: Homebrew on Linux gets quadrule too.
- Bad: the release must also attach plain `.tar.gz` builds, and there is a second recipe to keep. Not chosen.

### A formula built from source

- Good: no Gatekeeper warning; Mac and Linux.
- Bad: a 5 to 10 minute compile on every install, a bare `quadrule` command with no `.app`, and Homebrew's Rust in
  place of the pinned 1.99.1 (`rust-toolchain.toml`). Not chosen.

### A tap for quadrule alone, `tholent/homebrew-quadrule` (overruled)

- Good: quadrule's release job can write to nothing else.
- Bad: a new tap for every tool chris makes. chris's first answer, replaced by the shared tap the same day.

### A shared tap named exactly `tholent/homebrew`

- Good: the name chris first asked for.
- Bad: Homebrew looks for `homebrew-<name>`, so every user must type `brew tap tholent/homebrew
  https://github.com/tholent/homebrew` first, and `brew install tholent/homebrew/quadrule` alone fails. Not chosen.

### The tap inside this repository

- Good: no second repository and no deploy key.
- Bad: users must type the repository's URL when tapping, and tapping downloads all its history (`.git` is 55 MB).
  Not chosen.

### Clear the quarantine mark on install

- Good: the app opens with no warning.
- Bad: it quietly bypasses macOS's check on the user's behalf. Not chosen.

## Consequences

- The README's Download section gains the Homebrew lines; DEVELOPMENT.md's "Releases and packages" names the tap,
  the template and the secret.
- The deploy key can write to the whole shared tap, not only quadrule's cask; the job writes only
  `Casks/quadrule.rb`, and retries once on top of the tap's newest commit if another push got there first.
- chris owns two one-time steps: the tap repository and the deploy key with its secret. Until the secret exists the
  "Homebrew tap" job warns `TAP … status=skip reason=no-deploy-key` and the release still succeeds.
- `brew upgrade --cask` follows every published release, pre-releases included (the cask's `livecheck` uses
  `github_releases`, since every 0.x release is a pre-release).
- The tap's own CI audits the cask and installs it on a Mac after each push, so a broken cask shows there.

**Confirmed by:** the tap's CI passing after a release moves the cask, and `brew install --cask
tholent/tap/quadrule` opening the app on chris's Mac.

**Revisit when:** the app is signed and notarized (spec §15.4), which opens Homebrew's main collection
(`brew install --cask quadrule`), or Homebrew changes how taps may ship unsigned apps.

## History

- 2026-11-23: drafted from chris's three answers in the session that built it (light route; no question record).
- 2026-11-23 (still proposed): the tap moved from `tholent/homebrew-quadrule` to the shared `tholent/homebrew-tap`, at
  chris's word ("I would like there to be a tholent/homebrew repository that holds all my casks/formulas"; asked the
  name, he chose "tholent/homebrew-tap (Recommended)"). The quadrule-only tap is kept as an overruled alternative.
- 2026-11-23: ratified by chris ("ratify 025"). Earlier values, word for word: `status: proposed`;
  `status-note: drafted 2026-11-23 from chris's answers (\"Mac app cask\", \"New repo tholent/homebrew-quadrule\", \"Leave it, print steps\"), the tap renamed by his \"tholent/homebrew-tap\"; awaiting ratification`.
- 2026-11-24: landed. chris added the `HOMEBREW_TAP_DEPLOY_KEY` secret; the release job's tap step pushed quadrule
  0.16.6 to the tap. The tap's audit then needed two fixes (livecheck keeping pre-releases in
  `packaging/homebrew/quadrule.rb`; the tap's `audit_exceptions/github_prerelease_allowlist.json`), and its check now
  caches Homebrew's gems and downloads; tholent/homebrew-tap run 45370416264 passed style, audit, install and
  uninstall (`docs/STATUS-log.md`). Earlier values, word for word: `built: partial`; `proof: ".github/workflows/release.yml"`;
  `owed: "the tap repository tholent/homebrew-tap with quadrule's cask, and the HOMEBREW_TAP_DEPLOY_KEY secret (docs/STATUS-debt.md, ADR-025)"`.
