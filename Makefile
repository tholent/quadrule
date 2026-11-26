# Developer shortcuts for quadrule (bbq record docs/bbq/2026-11-10-makefile.md).
#
# `make` lists every target. A convenience for Linux and macOS: CI calls cargo directly (.github/workflows/ci.yml)
# and does not use this file. Each target is a thin wrapper, so you can always run the cargo command it prints.
#
# Every target covers the whole workspace, exactly as CI does (ADR-008, amendment of 2026-11-20). The `-all` forms
# are the same targets under their older names. Run `make check` (or `make check-all`) before pushing.

SHELL := bash
.SHELLFLAGS := -o pipefail -ec
.DEFAULT_GOAL := help

# Build jobs follow the computer: one per core, but no more than one per 1.5 GB of memory, as linking the test programs
# takes a lot of it (chris's 12-core, 32 GB VM gets 12; ADR-006, amendment of 2026-11-20). `make test JOBS=4` to change.
ifndef JOBS
JOBS := $(shell c=$$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2); \
  m=$$(awk '/^MemTotal:/ {print int($$2 / 1024)}' /proc/meminfo 2>/dev/null || \
       echo $$(( $$(sysctl -n hw.memsize 2>/dev/null || echo 3221225472) / 1048576 ))); \
  j=$$(( m / 1536 )); [ $$j -lt 1 ] && j=1; [ $$j -gt $$c ] && j=$$c; echo $$j)
endif
export CARGO_BUILD_JOBS := $(JOBS)

LOG := target/test.log

# For `run`, `lint`: FILE=path/to/page.quad. For `bench-drag`: FRAMES=150; SHAPES=1 runs it over the shapes page (every
# new kind), TRANSFORMS=1 over the transforms page (turned shapes, lines every way, opacity; `bench-rotate` always), GROUPS=1 over the groups page (the drag moves its heavy group), DEVICES=1 over the overrides page, LANDSCAPE=1 over the landscape page, PATTERNS=1 over the pattern page, REPEATS=1 over the repeat page, ANGLE=1 over the pattern-angle page, BORDERED=1 over the pattern-borders page, LINT=1 over the lint page (objects that make each gray, text-size, handwriting, dark-fill and thin-line rule fire), PAPER=1 over the paper page (ISO A4 at 300 PPI and ISO A5 at 600), LARGE=1 over the large paper page (ISO A3 and Tabloid at 600 PPI), FILE=path/to/starter.quad over a file, DEVICE=nomad|manta|<profile id> on that tablet, BASICTAB=1 with the Basic tab open and the whole window frame timed, TURNED=1 over the turned-patterns page (`bench-rotate` then turns its 45 degree grid), FONTS=1 over the fonts page, TEXTBOX=1 over the text-box page, MISSING=1 over the missing-fonts page (the text-box page in a font this computer lacks), NOMAD=1 on the Nomad (a drag then writes overrides), NOSTRIP=1 with the device previews strip folded shut (it is open in a bench otherwise); `bench-draw` also takes TOOL=rect|ellipse|polygon|pattern|line|text. For `calibration-sheet` and
# `safe-zones`: PROFILE=supernote.a6x2 (default all). For `fuzz`: SECS=60.
FILE ?=
FRAMES ?= 150
SHAPES ?=
TRANSFORMS ?=
GROUPS ?=
LANDSCAPE ?=
PATTERNS ?=
TURNED ?=
ANGLE ?=
BORDERED ?=
PAPER ?=
LINT :=
REPEATS ?=
FONTS ?=
TEXTBOX ?=
MISSING ?=
TOOL ?=
PROFILE ?= all
OUT ?= calib
SECS ?= 60
# What `make review` runs (check and check-all are the same since ADR-008's amendment of 2026-11-20).
CHECK ?= check-all
# `make build-proof` fails when target/ is bigger than this, in GiB, counted as file lengths the way `make prune` counts
# them (docs/proposals/build-pruning-2026-11-19.md, amended 2026-11-26).
TARGET_CAP_GB ?= 60
# For `prune`: DRY=1 lists what would go and deletes nothing.
DRY ?=
# For `starters`: STARTERS_CHECK=1 writes nothing and fails when a checked-in starter differs from the generator.
STARTERS_CHECK ?=

.PHONY: help build build-all fmt fmt-check clippy clippy-all test test-all test-v check check-all \
        run run-release bench-drag bench-draw bench-type bench-rotate bench-zoom bench-check bench-record calibration-sheet safe-zones starters lint fuzz build-proof review prune check-github

help: ## List the targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  make %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo
	@echo "Variables: JOBS=$(JOBS) FILE= FRAMES=$(FRAMES) PROFILE=$(PROFILE) OUT=$(OUT) SECS=$(SECS) CHECK=$(CHECK) TARGET_CAP_GB=$(TARGET_CAP_GB) DRY="

# ---- build, format, lint -------------------------------------------------------------------------------------

build: ## Build the whole workspace, window included (as CI)
	cargo build --workspace

build-all: build ## The same as build

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Check formatting (CI's rustfmt step)
	cargo fmt --all --check

clippy: ## Clippy on the whole workspace, warnings are errors (CI's clippy step)
	cargo clippy --workspace --all-targets -- -D warnings

clippy-all: clippy ## The same as clippy

# ---- tests ---------------------------------------------------------------------------------------------------
# Full output goes to $(LOG); the screen gets one line:
#   TESTS scope=workspace checks=<CHECK lines> check_fail=<result=fail> passed=<n> failed=<n> status=<ok|fail> runner=<nextest|cargo>
# The line is printed whether the tests passed or not; the exit status is the tests' own. With cargo-nextest installed
# (DEVELOPMENT.md) the tests run as separate processes, many at once (.config/nextest.toml); without it, `cargo test`.
# CHECK lines are counted wherever they sit: under `cargo test --nocapture` one can share a line with the runner's own
# output (docs/STATUS-debt.md).

define run_tests
@mkdir -p target
@scope='$(1)'; flags='$(2)'; status=ok; \
 if cargo nextest --version > /dev/null 2>&1; then \
   runner=nextest; cargo nextest run $$flags --no-fail-fast > $(LOG) 2>&1 || status=fail; \
   summary=$$(grep -E '^ *Summary \[' $(LOG) | tail -1); \
   passed=$$(echo "$$summary" | sed -nE 's/.*[: ] ([0-9]+) passed.*/\1/p'); \
   failed=$$(echo "$$summary" | sed -nE 's/.*[ ,] ([0-9]+) failed.*/\1/p'); \
 else \
   runner=cargo; cargo test $$flags -- --nocapture > $(LOG) 2>&1 || status=fail; \
   passed=$$(awk '/^test result:/ {n += $$4} END {print n + 0}' $(LOG)); \
   failed=$$(awk '/^test result:/ {n += $$6} END {print n + 0}' $(LOG)); \
 fi; \
 checks=$$(grep -o 'CHECK crate=' $(LOG) | wc -l | tr -d ' '); \
 cfail=$$(grep -o 'CHECK crate=.*' $(LOG) | grep -c 'result=fail' || true); \
 echo "TESTS scope=$$scope checks=$$checks check_fail=$$cfail passed=$${passed:-0} failed=$${failed:-0} status=$$status runner=$$runner"; \
 if [ "$$status" = fail ]; then \
   echo "--- failing output (full log: $(LOG)) ---"; \
   grep -E '^(error|test .* FAILED|failures:|---- )|^ *(FAIL|SIGSEGV|SIGABRT|TIMEOUT) \[|panicked|result=fail' $(LOG) | head -40; \
   exit 1; \
 fi
endef

test: ## Test the whole workspace (as CI); one summary line, full log in target/test.log
	$(call run_tests,workspace,--workspace)

test-all: test ## The same as test

test-v: ## Test the whole workspace with all output on screen (use `make test-v ARGS=name` to filter)
	cargo test --workspace -- --nocapture $(ARGS)

# ---- check: what to run before pushing -----------------------------------------------------------------------

# After the tests pass, both prune the build folder (`make prune`), except on CI, whose machines start clean. They run
# the xtask program the tests just built (`cargo test` leaves it at target/debug/xtask): `cargo xtask` would first
# build our crates another way (other library features), one more copy for prune to count.
define auto_prune
@if [ -n "$$CI" ]; then echo "PRUNE skipped: CI"; else target/debug/xtask prune; fi
endef

check: fmt-check clippy test ## fmt + clippy + tests on the whole workspace (what CI runs), then prune
	$(auto_prune)

check-all: check ## The same as check

# ---- the app -------------------------------------------------------------------------------------------------

run: ## Open the viewer/editor (FILE=page.quad, or the sample page)
	cargo run -p quadrule-app -- $(FILE)

run-release: ## The same, as a release build (what the speed numbers use)
	cargo run --release -p quadrule-app -- $(FILE)

bench-drag: ## Drag-speed bench on the sample page, release build (FRAMES=150, SHAPES=1 for the shapes page, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, PATTERNS=1, REPEATS=1, NOMAD=1, NOSTRIP=1); the 90th percentile is the judge
	cargo run --release -p quadrule-app -- --bench-drag $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(PATTERNS),--patterns-page) $(if $(REPEATS),--repeats-page) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(BORDERED),--bordered-page) $(if $(LINT),--lint-page) $(if $(PAPER),--paper-page) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(MISSING),--missing-fonts-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip) $(if $(LARGE),--large-paper-page) $(if $(BASICTAB),--basic-tab) $(if $(DEVICE),--device $(DEVICE)) $(FILE)

bench-draw: ## Drawing-speed bench: drags a pattern region out over the sample page, release build (FRAMES=150, SHAPES=1, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, PATTERNS=1, REPEATS=1, NOMAD=1, NOSTRIP=1, TOOL=rect|ellipse|polygon)
	cargo run --release -p quadrule-app -- --bench-draw $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(PATTERNS),--patterns-page) $(if $(REPEATS),--repeats-page) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(BORDERED),--bordered-page) $(if $(LINT),--lint-page) $(if $(PAPER),--paper-page) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(MISSING),--missing-fonts-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip) $(if $(LARGE),--large-paper-page) $(if $(BASICTAB),--basic-tab) $(if $(DEVICE),--device $(DEVICE)) $(if $(TOOL),--bench-tool $(TOOL)) $(FILE)

bench-type: ## Typing-speed bench: types into a long text box, release build (FRAMES=150, FONTS=1, TEXTBOX=1, NOSTRIP=1)
	cargo run --release -p quadrule-app -- --bench-type $(FRAMES) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(NOSTRIP),--no-strip)

bench-zoom: ## Zoom-speed bench: zooms the page in and out like a pinch, release build (FRAMES=150, GROUPS=1 for the groups page, BORDERED=1, NOSTRIP=1)
	cargo run --release -p quadrule-app -- --bench-zoom $(FRAMES) $(if $(GROUPS),--groups-page) $(if $(BORDERED),--bordered-page) $(if $(NOSTRIP),--no-strip)

bench-rotate: ## Rotate-speed bench: turns the transforms page's big rectangle with its handle, release build (FRAMES=150, TURNED=1 turns the turned-patterns page's grid)
	cargo run --release -p quadrule-app -- --bench-rotate $(FRAMES) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(NOSTRIP),--no-strip)

# The speed reference (rung 17, Q2): a fixed set of benches in a release build, each one's 90th percentile against the
# 50 ms line and against the numbers recorded for this computer in docs/bench/<hostname>.txt. A window session is needed
# (DISPLAY=:0 over ssh). BENCH_RUNS=2 runs each bench that many times and keeps the best; BENCH_ONLY=zoom runs only the
# benches with that text in their name (never with bench-record). One BENCH line per bench and a BENCH-CHECK line.
BENCH_RUNS ?= 2
BENCH_ONLY ?=

bench-check: ## The speed reference: every bench's 90th percentile under 50 ms and within 20 % of docs/bench/<this computer>.txt (BENCH_RUNS=2, BENCH_ONLY=text); needs a window session
	cargo build --release -p quadrule-app
	cargo xtask bench-check --runs $(BENCH_RUNS) $(if $(BENCH_ONLY),--only $(BENCH_ONLY))

bench-record: ## Run the same set and write the numbers for this computer to docs/bench/<hostname>.txt (refuses while any bench is over 50 ms)
	cargo build --release -p quadrule-app
	cargo xtask bench-check --record --runs $(BENCH_RUNS)

# ---- maintainer tasks ----------------------------------------------------------------------------------------

calibration-sheet: ## Write the calibration pages (PROFILE=all, OUT=calib)
	cargo xtask calibration-sheet --profile $(PROFILE) --out $(OUT)

safe-zones: ## Print the safe zones measured from the device screenshots (PROFILE=all)
	cargo xtask safe-zones --profile $(PROFILE)

starters: ## Write the starter pages to crates/ui/starters/ from code (STARTERS_CHECK=1 only compares them)
	cargo xtask starters $(if $(STARTERS_CHECK),--check)

lint: ## Lint a document (FILE=page.quad, or the sample page)
	cargo xtask lint $(FILE)

prune: ## Delete old copies of our own builds (keeps 2 of each test program, 4 of each library and program) and incremental caches unused for 3 days from target/ (DRY=1 to list only); one PRUNE line
	cargo build --quiet --bin xtask && target/debug/xtask prune $(if $(DRY),--dry-run)

fuzz: ## Fuzz the .quad reader and the profile loader for SECS=60 each (needs nightly Rust and cargo-fuzz)
	cd crates/format && mkdir -p fuzz/corpus/open_quad fuzz/corpus/from_ron && \
	cargo +nightly fuzz run open_quad fuzz/corpus/open_quad fuzz/seeds/open_quad -- -max_total_time=$(SECS) && \
	cargo +nightly fuzz run from_ron fuzz/corpus/from_ron fuzz/seeds/from_ron -- -max_total_time=$(SECS)

# ---- review --------------------------------------------------------------------------------------------------

check-github: ## Read the repository's GitHub rules back against ADR-022 (rulesets, secret scanning, push protection): one GITHUB line; needs gh
	@.github/scripts/check-github-rules.sh

build-proof: ## Prove the build on disk matches HEAD: one BUILD line (rebuilt=0 means nothing was stale); fails over TARGET_CAP_GB
	@head=$$(git rev-parse --short HEAD); \
	 dirty=$$(if [ -n "$$(git status --porcelain)" ]; then echo yes; else echo no; fi); \
	 mkdir -p target; \
	 cargo build --workspace 2>&1 | tee target/build-proof.log | tail -n 3; \
	 rebuilt=$$(grep -c '^ *Compiling quadrule' target/build-proof.log || true); \
	 kb=$$( (du -sk --apparent-size target 2>/dev/null || du -skA target) | cut -f1); \
	 gb=$$(awk -v kb=$$kb 'BEGIN {printf "%.1f", kb / 1048576}'); \
	 echo "BUILD head=$$head dirty=$$dirty rebuilt=$$rebuilt target_gb=$$gb"; \
	 if awk -v kb=$$kb -v cap=$(TARGET_CAP_GB) 'BEGIN {exit !(kb > cap * 1048576)}'; then \
	   echo "over the cap (TARGET_CAP_GB=$(TARGET_CAP_GB)): run make prune"; exit 1; \
	 fi

review: ## Everything a reviewer wants: BUILD line, check, commits and diff against origin/main
	@$(MAKE) --no-print-directory build-proof
	@$(MAKE) --no-print-directory $(CHECK)
	@echo "--- commits not on origin/main ---"
	@git log --oneline origin/main..HEAD
	@echo "--- files changed against origin/main ---"
	@git diff --stat origin/main
	@echo "--- uncommitted ---"
	@git status --short
