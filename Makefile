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

# The Pi has 4 cores and little memory; two build jobs by default. `make test JOBS=4` to change.
JOBS ?= 2
export CARGO_BUILD_JOBS := $(JOBS)

LOG := target/test.log

# For `run`, `lint`: FILE=path/to/page.quad. For `bench-drag`: FRAMES=150; SHAPES=1 runs it over the shapes page (every
# new kind), TRANSFORMS=1 over the transforms page (turned shapes, lines every way, opacity; `bench-rotate` always), GROUPS=1 over the groups page (the drag moves its heavy group), DEVICES=1 over the overrides page, LANDSCAPE=1 over the landscape page, NOMAD=1 on the Nomad (a drag then writes overrides), NOSTRIP=1 with the device previews strip folded shut (it is open in a bench otherwise); `bench-draw` also takes TOOL=rect|ellipse|polygon|pattern|line|text. For `calibration-sheet` and
# `safe-zones`: PROFILE=supernote.a6x2 (default all). For `fuzz`: SECS=60.
FILE ?=
FRAMES ?= 150
SHAPES ?=
TRANSFORMS ?=
GROUPS ?=
LANDSCAPE ?=
TOOL ?=
PROFILE ?= all
OUT ?= calib
SECS ?= 60
# What `make review` runs: check (fast) or check-all (CI parity).
CHECK ?= check-all

.PHONY: help build build-all fmt fmt-check clippy clippy-all test test-all test-v check check-all \
        run run-release bench-drag bench-draw bench-rotate calibration-sheet safe-zones lint fuzz build-proof review

help: ## List the targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  make %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo
	@echo "Variables: JOBS=$(JOBS) FILE= FRAMES=$(FRAMES) PROFILE=$(PROFILE) OUT=$(OUT) SECS=$(SECS) CHECK=$(CHECK)"

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
#   TESTS scope=<default|workspace> checks=<CHECK lines> check_fail=<result=fail> passed=<n> failed=<n> status=<ok|fail>
# The line is printed whether the tests passed or not; the exit status is the tests' own.

define run_tests
@mkdir -p target
@scope='$(1)'; flags='$(2)'; \
 status=ok; cargo test $$flags -- --nocapture > $(LOG) 2>&1 || status=fail; \
 checks=$$(grep -c '^CHECK' $(LOG) || true); \
 cfail=$$(grep '^CHECK' $(LOG) | grep -c 'result=fail' || true); \
 passed=$$(awk '/^test result:/ {n += $$4} END {print n + 0}' $(LOG)); \
 failed=$$(awk '/^test result:/ {n += $$6} END {print n + 0}' $(LOG)); \
 echo "TESTS scope=$$scope checks=$$checks check_fail=$$cfail passed=$$passed failed=$$failed status=$$status"; \
 if [ "$$status" = fail ]; then \
   echo "--- failing output (full log: $(LOG)) ---"; \
   grep -E '^(error|test .* FAILED|failures:|---- )|panicked|result=fail' $(LOG) | head -40; \
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

check: fmt-check clippy test ## fmt + clippy + tests on the default members (quick)

check-all: fmt-check clippy-all test-all ## fmt + clippy + tests on the whole workspace (what CI runs)

# ---- the app -------------------------------------------------------------------------------------------------

run: ## Open the viewer/editor (FILE=page.quad, or the sample page)
	cargo run -p quadrule-app -- $(FILE)

run-release: ## The same, as a release build (what the speed numbers use)
	cargo run --release -p quadrule-app -- $(FILE)

bench-drag: ## Drag-speed bench on the sample page, release build (FRAMES=150, SHAPES=1 for the shapes page, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, NOMAD=1, NOSTRIP=1); the 90th percentile is the judge
	cargo run --release -p quadrule-app -- --bench-drag $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip)

bench-draw: ## Drawing-speed bench: drags a pattern region out over the sample page, release build (FRAMES=150, SHAPES=1, TRANSFORMS=1, GROUPS=1, DEVICES=1, LANDSCAPE=1, NOMAD=1, NOSTRIP=1, TOOL=rect|ellipse|polygon)
	cargo run --release -p quadrule-app -- --bench-draw $(FRAMES) $(if $(SHAPES),--shapes-page) $(if $(TRANSFORMS),--transforms-page) $(if $(GROUPS),--groups-page) $(if $(DEVICES),--devices-page) $(if $(LANDSCAPE),--landscape-page) $(if $(NOMAD),--bench-secondary) $(if $(NOSTRIP),--no-strip) $(if $(TOOL),--bench-tool $(TOOL))

bench-rotate: ## Rotate-speed bench: turns the transforms page's big rectangle with its handle, release build (FRAMES=150)
	cargo run --release -p quadrule-app -- --bench-rotate $(FRAMES)

# ---- maintainer tasks ----------------------------------------------------------------------------------------

calibration-sheet: ## Write the calibration pages (PROFILE=all, OUT=calib)
	cargo xtask calibration-sheet --profile $(PROFILE) --out $(OUT)

safe-zones: ## Print the safe zones measured from the device screenshots (PROFILE=all)
	cargo xtask safe-zones --profile $(PROFILE)

lint: ## Lint a document (FILE=page.quad, or the sample page)
	cargo xtask lint $(FILE)

fuzz: ## Fuzz the .quad reader for SECS=60 (needs nightly Rust and cargo-fuzz)
	cd crates/format && mkdir -p fuzz/corpus/open_quad && \
	cargo +nightly fuzz run open_quad fuzz/corpus/open_quad fuzz/seeds/open_quad -- -max_total_time=$(SECS)

# ---- review --------------------------------------------------------------------------------------------------

build-proof: ## Prove the build on disk matches HEAD: one BUILD line (rebuilt=0 means nothing was stale)
	@head=$$(git rev-parse --short HEAD); \
	 dirty=$$(if [ -n "$$(git status --porcelain)" ]; then echo yes; else echo no; fi); \
	 mkdir -p target; \
	 cargo build --workspace 2>&1 | tee target/build-proof.log | tail -n 3; \
	 rebuilt=$$(grep -c '^ *Compiling quadrule' target/build-proof.log || true); \
	 echo "BUILD head=$$head dirty=$$dirty rebuilt=$$rebuilt"

review: ## Everything a reviewer wants: BUILD line, check (CHECK=check-all|check), commits and diff against origin/main
	@$(MAKE) --no-print-directory build-proof
	@$(MAKE) --no-print-directory $(CHECK)
	@echo "--- commits not on origin/main ---"
	@git log --oneline origin/main..HEAD
	@echo "--- files changed against origin/main ---"
	@git diff --stat origin/main
	@echo "--- uncommitted ---"
	@git status --short
