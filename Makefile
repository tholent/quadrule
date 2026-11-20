# Developer shortcuts for quadrule (bbq record docs/bbq/2026-11-10-makefile.md).
#
# `make` lists every target. A convenience for Linux and macOS: CI calls cargo directly (.github/workflows/ci.yml)
# and does not use this file. Each target is a thin wrapper, so you can always run the cargo command it prints.
#
# Fast vs whole: plain `test`, `clippy` and `check` use the default members (Cargo.toml `default-members`: the
# graphics crates ui and app are left out, which keeps the Pi quick). The `-all` forms use --workspace, exactly
# as CI does. Run `make check-all` before pushing.

SHELL := bash
.SHELLFLAGS := -o pipefail -ec
.DEFAULT_GOAL := help

# Build jobs follow the computer: one per core, but no more than one per 1.5 GB of memory, as linking the test programs
# takes a lot of it (the Pi 5's 4 cores and 4 GB give 2, chris's 12-core VM 12). `make test JOBS=4` to change.
ifndef JOBS
JOBS := $(shell c=$$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2); \
  m=$$(awk '/^MemTotal:/ {print int($$2 / 1024)}' /proc/meminfo 2>/dev/null || \
       echo $$(( $$(sysctl -n hw.memsize 2>/dev/null || echo 3221225472) / 1048576 ))); \
  j=$$(( m / 1536 )); [ $$j -lt 1 ] && j=1; [ $$j -gt $$c ] && j=$$c; echo $$j)
endif
export CARGO_BUILD_JOBS := $(JOBS)

LOG := target/test.log

# For `run`, `lint`: FILE=path/to/page.quad. For `bench-drag`: FRAMES=150; SHAPES=1 runs it over the shapes page (every
# new kind), TRANSFORMS=1 over the transforms page (turned shapes, lines every way, opacity; `bench-rotate` always), GROUPS=1 over the groups page (the drag moves its heavy group), DEVICES=1 over the overrides page, LANDSCAPE=1 over the landscape page, PATTERNS=1 over the pattern page, REPEATS=1 over the repeat page, ANGLE=1 over the pattern-angle page, BORDERED=1 over the pattern-borders page, LINT=1 over the lint page (objects that make each gray, text-size, handwriting, dark-fill and thin-line rule fire), PAPER=1 over the paper page (ISO A4 at 300 PPI and ISO A5 at 600), TURNED=1 over the turned-patterns page (`bench-rotate` then turns its 45 degree grid), FONTS=1 over the fonts page, TEXTBOX=1 over the text-box page, MISSING=1 over the missing-fonts page (the text-box page in a font this computer lacks), NOMAD=1 on the Nomad (a drag then writes overrides), NOSTRIP=1 with the device previews strip folded shut (it is open in a bench otherwise); `bench-draw` also takes TOOL=rect|ellipse|polygon|pattern|line|text. For `calibration-sheet` and
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
# What `make review` runs: check (fast) or check-all (CI parity).
CHECK ?= check-all
# `make build-proof` fails when target/ is bigger than this, in GiB (docs/proposals/build-pruning-2026-11-19.md).
TARGET_CAP_GB ?= 50
# For `prune`: DRY=1 lists what would go and deletes nothing.
DRY ?=

.PHONY: help build build-all fmt fmt-check clippy clippy-all test test-all test-v check check-all \
        run run-release bench-drag bench-draw bench-type bench-rotate bench-zoom calibration-sheet safe-zones lint fuzz build-proof review prune

help: ## List the targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  make %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo
	@echo "Variables: JOBS=$(JOBS) FILE= FRAMES=$(FRAMES) PROFILE=$(PROFILE) OUT=$(OUT) SECS=$(SECS) CHECK=$(CHECK) TARGET_CAP_GB=$(TARGET_CAP_GB) DRY="

# ---- build, format, lint -------------------------------------------------------------------------------------

build: ## Build the default members (no window)
	cargo build

build-all: ## Build the whole workspace, window included (as CI)
	cargo build --workspace

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Check formatting (CI's rustfmt step)
	cargo fmt --all --check

clippy: ## Clippy on the default members, warnings are errors
	cargo clippy --all-targets -- -D warnings

clippy-all: ## Clippy on the whole workspace (CI's clippy step)
	cargo clippy --workspace --all-targets -- -D warnings

# ---- tests ---------------------------------------------------------------------------------------------------
# Full output goes to $(LOG); the screen gets one line:
#   TESTS scope=<default|workspace> checks=<CHECK lines> check_fail=<result=fail> passed=<n> failed=<n> status=<ok|fail> runner=<nextest|cargo>
# The line is printed whether the tests passed or not; the exit status is the tests' own. With cargo-nextest installed
# (DEVELOPMENT.md) the tests run as separate processes, many at once (.config/nextest.toml); without it, `cargo test`.

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
 checks=$$(grep -c '^ *CHECK' $(LOG) || true); \
 cfail=$$(grep '^ *CHECK' $(LOG) | grep -c 'result=fail' || true); \
 echo "TESTS scope=$$scope checks=$$checks check_fail=$$cfail passed=$${passed:-0} failed=$${failed:-0} status=$$status runner=$$runner"; \
 if [ "$$status" = fail ]; then \
   echo "--- failing output (full log: $(LOG)) ---"; \
   grep -E '^(error|test .* FAILED|failures:|---- )|^ *(FAIL|SIGSEGV|SIGABRT|TIMEOUT) \[|panicked|result=fail' $(LOG) | head -40; \
   exit 1; \
 fi
endef

test: ## Test the default members; one summary line, full log in target/test.log
	$(call run_tests,default,)

test-all: ## Test the whole workspace (as CI); one summary line
	$(call run_tests,workspace,--workspace)

test-v: ## Test the default members with all output on screen (use `make test-v ARGS=name` to filter)
	cargo test -- --nocapture $(ARGS)

# ---- check: what to run before pushing -----------------------------------------------------------------------

# After the tests pass, both prune the build folder (`make prune`), except on CI, whose machines start clean. They run
# the xtask program the tests just built (`cargo test` leaves it at target/debug/xtask): `cargo xtask` would first
# build our crates another way (other library features), one more copy for prune to count.
define auto_prune
@if [ -n "$$CI" ]; then echo "PRUNE skipped: CI"; else target/debug/xtask prune; fi
endef

check: fmt-check clippy test ## fmt + clippy + tests on the default members (quick), then prune
	$(auto_prune)

check-all: fmt-check clippy-all test-all ## fmt + clippy + tests on the whole workspace (what CI runs), then prune
	$(auto_prune)

# ---- the app -------------------------------------------------------------------------------------------------

run: ## Open the viewer/editor (FILE=page.quad, or the sample page)
	cargo run -p quadrule-app -- $(FILE)

run-release: ## The same, as a release build (what the speed numbers use)
	cargo run --release -p quadrule-app -- $(FILE)

bench-drag: ## Drag-speed bench on the sample page, release build (FRAMES=150, SHAPES=1 for the shapes page, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, PATTERNS=1, REPEATS=1, NOMAD=1, NOSTRIP=1); the 90th percentile is the judge
	cargo run --release -p quadrule-app -- --bench-drag $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(PATTERNS),--patterns-page) $(if $(REPEATS),--repeats-page) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(BORDERED),--bordered-page) $(if $(LINT),--lint-page) $(if $(PAPER),--paper-page) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(MISSING),--missing-fonts-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip)

bench-draw: ## Drawing-speed bench: drags a pattern region out over the sample page, release build (FRAMES=150, SHAPES=1, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, PATTERNS=1, REPEATS=1, NOMAD=1, NOSTRIP=1, TOOL=rect|ellipse|polygon)
	cargo run --release -p quadrule-app -- --bench-draw $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(PATTERNS),--patterns-page) $(if $(REPEATS),--repeats-page) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(BORDERED),--bordered-page) $(if $(LINT),--lint-page) $(if $(PAPER),--paper-page) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(MISSING),--missing-fonts-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip) $(if $(TOOL),--bench-tool $(TOOL))

bench-type: ## Typing-speed bench: types into a long text box, release build (FRAMES=150, FONTS=1, TEXTBOX=1, NOSTRIP=1)
	cargo run --release -p quadrule-app -- --bench-type $(FRAMES) $(if $(FONTS),--fonts-page) $(if $(TEXTBOX),--textbox-page) $(if $(NOSTRIP),--no-strip)

bench-zoom: ## Zoom-speed bench: zooms the page in and out like a pinch, release build (FRAMES=150, GROUPS=1 for the groups page, BORDERED=1, NOSTRIP=1)
	cargo run --release -p quadrule-app -- --bench-zoom $(FRAMES) $(if $(GROUPS),--groups-page) $(if $(BORDERED),--bordered-page) $(if $(NOSTRIP),--no-strip)

bench-rotate: ## Rotate-speed bench: turns the transforms page's big rectangle with its handle, release build (FRAMES=150, TURNED=1 turns the turned-patterns page's grid)
	cargo run --release -p quadrule-app -- --bench-rotate $(FRAMES) $(if $(TURNED),--turned-patterns-page) $(if $(ANGLE),--pattern-angle-page) $(if $(NOSTRIP),--no-strip)

# ---- maintainer tasks ----------------------------------------------------------------------------------------

calibration-sheet: ## Write the calibration pages (PROFILE=all, OUT=calib)
	cargo xtask calibration-sheet --profile $(PROFILE) --out $(OUT)

safe-zones: ## Print the safe zones measured from the device screenshots (PROFILE=all)
	cargo xtask safe-zones --profile $(PROFILE)

lint: ## Lint a document (FILE=page.quad, or the sample page)
	cargo xtask lint $(FILE)

prune: ## Delete old copies of our own builds (keeps 2 of each test program, 4 of each library and program) and incremental caches unused for 3 days from target/ (DRY=1 to list only); one PRUNE line
	cargo build --quiet --bin xtask && target/debug/xtask prune $(if $(DRY),--dry-run)

fuzz: ## Fuzz the .quad reader and the profile loader for SECS=60 each (needs nightly Rust and cargo-fuzz)
	cd crates/format && mkdir -p fuzz/corpus/open_quad fuzz/corpus/from_ron && \
	cargo +nightly fuzz run open_quad fuzz/corpus/open_quad fuzz/seeds/open_quad -- -max_total_time=$(SECS) && \
	cargo +nightly fuzz run from_ron fuzz/corpus/from_ron fuzz/seeds/from_ron -- -max_total_time=$(SECS)

# ---- review --------------------------------------------------------------------------------------------------

build-proof: ## Prove the build on disk matches HEAD: one BUILD line (rebuilt=0 means nothing was stale); fails over TARGET_CAP_GB
	@head=$$(git rev-parse --short HEAD); \
	 dirty=$$(if [ -n "$$(git status --porcelain)" ]; then echo yes; else echo no; fi); \
	 mkdir -p target; \
	 cargo build --workspace 2>&1 | tee target/build-proof.log | tail -n 3; \
	 rebuilt=$$(grep -c '^ *Compiling quadrule' target/build-proof.log || true); \
	 kb=$$(du -sk target | cut -f1); \
	 gb=$$(awk -v kb=$$kb 'BEGIN {printf "%.1f", kb / 1048576}'); \
	 echo "BUILD head=$$head dirty=$$dirty rebuilt=$$rebuilt target_gb=$$gb"; \
	 if awk -v kb=$$kb -v cap=$(TARGET_CAP_GB) 'BEGIN {exit !(kb > cap * 1048576)}'; then \
	   echo "over the cap (TARGET_CAP_GB=$(TARGET_CAP_GB)): run make prune"; exit 1; \
	 fi

review: ## Everything a reviewer wants: BUILD line, check (CHECK=check-all|check), commits and diff against origin/main
	@$(MAKE) --no-print-directory build-proof
	@$(MAKE) --no-print-directory $(CHECK)
	@echo "--- commits not on origin/main ---"
	@git log --oneline origin/main..HEAD
	@echo "--- files changed against origin/main ---"
	@git diff --stat origin/main
	@echo "--- uncommitted ---"
	@git status --short
