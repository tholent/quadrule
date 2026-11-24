---
status: proposed
status-note: "drafted 2026-11-23 from chris's answers (\"Mac app cask\", \"New repo tholent/homebrew-quadrule\", \"Leave it, print steps\"); awaiting ratification"
owner: chris
kind: process
built: partial
proof: ".github/workflows/release.yml"
owed: "the tap repository tholent/homebrew-quadrule with its first cask, and the HOMEBREW_TAP_DEPLOY_KEY secret (docs/STATUS-debt.md, ADR-025)"
supersedes: nothing
amends: nothing
---
# ADR-025 — Quadrule installs on macOS with Homebrew, as a cask in quadrule's own tap that follows each release

## Decision

1. **A cask, Mac only.** `brew install --cask tholent/quadrule/quadrule` puts the release's universal `.dmg`'s
   `Quadrule.app` in Applications. There is no Homebrew formula (command-line build) and nothing for Homebrew on
   Linux.
2. **Its own tap, `github.com/tholent/homebrew-quadrule`.** The cask lives at `Casks/quadrule.rb` there, not in
   Homebrew's main collection and not in this repository.
3. **The release moves it.** The cask's source is `packaging/homebrew/quadrule.rb` in this repository. After a
   release is published, `.github/workflows/release.yml` (job "Homebrew tap") fills in the version and the `.dmg`'s
   sha256 and pushes it to the tap, with a deploy key that can write to the tap only (secret
   `HOMEBREW_TAP_DEPLOY_KEY`). A `-test` tag never moves it.
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
the app is signed.

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
- chris owns two one-time steps: the tap repository and the deploy key with its secret. Until the secret exists the
  "Homebrew tap" job warns `TAP … status=skip reason=no-deploy-key` and the release still succeeds.
- `brew upgrade --cask` follows every published release, pre-releases included (the cask's `livecheck` uses
  `github_releases`, since every 0.x release is a pre-release).
- The tap's own CI audits the cask and installs it on a Mac after each push, so a broken cask shows there.

**Confirmed by:** the tap's CI passing after a release moves the cask, and `brew install --cask
tholent/quadrule/quadrule` opening the app on chris's Mac.

**Revisit when:** the app is signed and notarized (spec §15.4), which opens Homebrew's main collection
(`brew install --cask quadrule`), or Homebrew changes how taps may ship unsigned apps.

## History

- 2026-11-23: drafted from chris's three answers in the session that built it (light route; no question record).
