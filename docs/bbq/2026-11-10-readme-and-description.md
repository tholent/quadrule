# bbq record — README, DEVELOPMENT.md and the GitHub description

2026-11-10 · untyped bbq · 12 questions (8 asked for, 4 more taken when the owed ones were asked now) · lead: this
session (Sonnet 5.5), run here; a small job, one sitting.

Grounding: `STATUS.md`, `docs/STATUS-log.md` (last entry), `napkins/quadrule-spec.md` §1, §3, §8, §11, §13, §16,
`Cargo.toml`, `rust-toolchain.toml`, `Makefile`, `.github/workflows/ci.yml`, `docs/calibration.md`,
`docs/device-measurements/README.md`, `crates/profiles/data/`. `gh repo view` showed: public, Apache-2.0, issues on,
no description, no topics, no homepage. No README existed.

## Rules

- **Q1 Count.** 8 questions (four more were then taken, see Q9–Q12).
- **Q2 Lead.** This session, run here.
- **Q3 Audience.** Users first. chris's words: "let's do a README.md that focuses on users first and a DEVELOPMENT.md
  that focuses on developer setup and how to contribute". This grew the job from one file to two; said so aloud.
- **Q4 State.** A pre-alpha banner plus a short "what works today" list, linking `STATUS.md`.
- **Q5 GitHub description.** "Design pixel-perfect e-ink page templates for Supernote tablets".
- **Q6 README sections.** Features / what makes it e-ink-aware; run it and build/test; licence and contributing note.
- **Q7 Contributing.** Issues and bug reports welcome; open an issue before a pull request.
- **Q8 Protect.** Nothing more.
- **Q9 Topics.** `supernote`, `e-ink`, `vector-graphics`, `design-tool`, `templates`, `wallpaper`. No homepage.
- **Q10 Pictures.** None yet; leave a note to add later.
- **Q11 Affiliation.** One line near the end: independent, not affiliated with or endorsed by Ratta or Supernote.
- **Q12 DEVELOPMENT.md.** Setup, build and test; crate map; contributing process; calibration and device
  measurement how-to.
- **Landing.** Commit and push to `main` (`AGENTS.md`, amendment of 2026-11-09). Settled by that amendment, not asked.

## Owed

None.

## Defaulted

- The README says what works today and what does not (no template gallery, no wallpapers, no prebuilt downloads:
  build from source), taken from `STATUS.md` and spec §16. veto: say so.
- The README's words are my proposal; wording and tone are chris's to rule (`AGENTS.md`). veto: say so.
- Linux needs no extra packages beyond what GitHub's stock Ubuntu runner has (`.github/workflows/ci.yml` installs
  none); DEVELOPMENT.md says to install fontconfig's development files if the build asks. veto: say so.

## Kickoff

Run here, now. Write `README.md` and `DEVELOPMENT.md`, set the GitHub description and topics with `gh repo edit`,
add a pointer in `AGENTS.md`'s "Where things live", update `STATUS.md` and `docs/STATUS-log.md`, commit and push to
`main`.
