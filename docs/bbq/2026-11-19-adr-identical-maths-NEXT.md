# Next agent's prompt — quadrule ADR-017: the same maths on every computer

2026-11-19 · written by the lead that drafted ADR-017 (Opus 5.5, high) at chris's word ("ADR + next-agent prompt").
Paste this into a fresh session in the quadrule folder.

## Who

**Opus 5.5 at high effort** builds it, in one session on `main` (AGENTS.md, amended 2026-11-09). The work is small in
code but fiddly in linking, and needs judgement when a system refuses (tier chosen at the bbq's last question). A
**different tier checks it**: before landing, run `/code-review high` with a Sonnet 5.5 reviewer (or spawn one) on the
diff, and chris looks at any new golden images.

## Read first

- `docs/decisions/ADR-017-same-maths-on-every-computer.md` (ratified by chris 2026-11-19).
- `docs/bbq/2026-11-19-adr-identical-maths.md`: the premises P1 to P8, chris's words, the Defaulted table.
- `docs/decisions/ADR-003-export-renderer.md` (the banner at item 2) and `ADR-006-checks-and-ci.md`.
- AGENTS.md. Start with `make build-proof` and say the BUILD line.

**If this session runs on chris's VM** (Intel Xeon E5-2470 v2, 12 threads, no FMA; `grep -c fma /proc/cpuinfo` prints
0): the golden tests **fail there before the fix**, as the record says (P1); that is expected, not your doing. After
the fix they must pass there with no change to the golden images, which makes the VM the local proof. Build with
`make … JOBS=12` (the Makefile defaults to 2 for the Pi), and run tests with `--no-fail-fast` so one failing test
program doesn't hide the rest.

## Budget

No share set: chris lifted this week's hard limits and said "I have a free reset this week, so I am grinding this out"
(`docs/bbq/2026-11-18-fixes-before-16b.md`, Budget). Report the usage meter at the start and end if chris gives a
reading. Stop and report at either stop condition below rather than spend past them.

## The first step only: confirm the cause in CI

Add a job to `.github/workflows/ci.yml`: Linux x86_64 (`ubuntu-latest`), runs `cargo test -p quadrule-render
--no-fail-fast -- --nocapture` with glibc told to act as if the processor lacked FMA and AVX2.

- Take the exact `GLIBC_TUNABLES` value from glibc's manual for the runner's glibc version (print `ldd --version` in
  the job), never from memory. Print what glibc used, e.g. `/lib64/ld-linux-x86-64.so.2 --list-diagnostics | grep -i
  -E 'fma|avx2'`, so the log shows FMA is off.
- Mark it `continue-on-error: true` for now, so `main` stays green (record, Defaulted).
- **The test it must pass, which is the red arm:** before any fix, this job must **fail** with the VM's checksums:
  `devices_golden profile=supernote.a6x2 … sha256=e2cc42fdd3c517a924af8ea93e261a5b0ece7180babd86652ed7421f7e0163cd`
  and `groups_golden profile=supernote.a6x2 … sha256=d78f33f29a37aaaa8967f033abfd7d9c2a61acba2557ffd4b79948b202a0789a`.
  - Both match: the cause is confirmed. Write it in the record's Amendments with the run's link, then go on to the
    rest of ADR-017's Decision (the crate, its routing test with its own red arm, the three systems, the job made
    required, golden images for chris's look).
  - It passes, or fails with other checksums: **stop**. The premise is wrong; report the run to chris before building
    anything.

## Must not touch

- kurbo, vello_cpu, vello: no patches and no version changes (ADR-017 item 2 allows a patch only as a fallback
  reported to chris first).
- The golden images and `SHA256SUMS`: re-bless only after chris has seen the changed-pixel count and a picture of
  where (ADR-017 item 3).
- `Math::Fastest` and `Math::Portable` stay as they are; ADR-003 is otherwise unchanged.
- The crate is `quadrule-maths` in `crates/maths/` (named by chris 2026-11-19); ADR-017 is ratified.

## Stop conditions

- A wrong premise (above), or a linker that refuses the app's own maths functions on any system: stop and report;
  the fallback is chris's to approve.
- A fix that spreads past a few files, or past the new crate plus the workflow, the Makefile and the docs: stop and
  report.

## Close

`docs/STATUS-log.md` entry with every number as the command that made it; `STATUS.md`; ADR-017's `built` and `proof`
(a test, never a commit); `DEVELOPMENT.md` (the crate, the FMA-off job); ask chris to rerun the golden tests on his VM
(`cargo test -p quadrule-render --no-fail-fast`).
