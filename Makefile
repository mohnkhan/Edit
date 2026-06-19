# Makefile for Linux EDIT.COM Clone
# Targets: build release check smoke perf-check static package-deb package-rpm docs-gate
#          stress-test ci-local tmpfs-setup tmpfs-status tmpfs-teardown help

BINARY     := edit
CARGO      := cargo
TARGET_DIR := target

# Per-checkout tmpfs root for the "Save your SSD" build redirect (see
# docs/dev-tmpfs.md). Hashed from the absolute repo path so multiple checkouts
# never share a build cache.
EDIT_TMPFS_HASH := $(shell printf '%s' "$(CURDIR)" | sha256sum | cut -c1-12)
EDIT_TMPFS_ROOT := /tmp/edit/$(EDIT_TMPFS_HASH)

.PHONY: build release check smoke perf-check static package-deb package-rpm docs-gate \
        stress-test ci-local tmpfs-setup tmpfs-status tmpfs-teardown help

build:
	$(CARGO) build

release:
	$(CARGO) build --release

check:
	$(CARGO) test

smoke:
	@if ! command -v expect > /dev/null 2>&1; then \
		echo "ERROR: 'expect' not installed — required for smoke tests"; exit 1; fi
	@for f in tests/smoke/*.exp; do \
		echo "Running $$f ..."; \
		LC_ALL=C.UTF-8 LANG=C.UTF-8 expect "$$f" || exit 1; \
	done
	@echo "All smoke tests passed."

perf-check:
	$(CARGO) bench 2>&1 | tee /tmp/edit-bench.log
	@echo "Benchmark results saved to /tmp/edit-bench.log"

static:
	$(CARGO) build --target x86_64-unknown-linux-musl --profile release-static

package-deb:
	cargo deb

package-rpm:
	$(CARGO) build --release
	rpmbuild -bb packaging/edit.spec

docs-gate:
	@echo "Checking docs gate..."
	@test -f CHANGELOG.md || (echo "ERROR: CHANGELOG.md missing" && exit 1)
	@test -f docs/STATUS.md || (echo "ERROR: docs/STATUS.md missing" && exit 1)
	@echo "Docs gate passed."

stress-test:
	EDIT_STRESS_DURATION_SECS=300 $(CARGO) test --test stress -- --nocapture

ci-local:
	$(CARGO) fmt --check
	$(CARGO) clippy -- -D warnings
	$(CARGO) test
	$(MAKE) smoke
	$(MAKE) perf-check

# ── Developer ergonomics: "Save your SSD" (opt-in; see docs/dev-tmpfs.md) ──────
# Redirect target/ into a per-checkout tmpfs subdir so Cargo's write-heavy
# incremental builds hit RAM instead of the SSD. Reversible, idempotent, no-op
# on CI.
tmpfs-setup:
	@if [ "$$CI" = "true" ]; then \
	  echo "[tmpfs-setup] CI detected; skipping (this is a dev-box knob)"; \
	  exit 0; \
	fi
	@bash scripts/tmpfs-setup.sh "$(EDIT_TMPFS_ROOT)"

tmpfs-status:
	@bash scripts/tmpfs-status.sh "$(EDIT_TMPFS_ROOT)"

tmpfs-teardown:
	@bash scripts/tmpfs-teardown.sh "$(EDIT_TMPFS_ROOT)" "$(WIPE)"

help:
	@echo "Available targets:"
	@echo "  build        Build debug binary"
	@echo "  release      Build release binary (stripped)"
	@echo "  check        Run unit and integration tests"
	@echo "  smoke        Run expect-based smoke tests (requires expect + tmux)"
	@echo "  perf-check   Run criterion benchmarks"
	@echo "  static       Build musl static binary for x86_64"
	@echo "  package-deb  Build .deb package via cargo-deb"
	@echo "  package-rpm  Build .rpm package via rpmbuild"
	@echo "  docs-gate    Verify CHANGELOG.md and docs/STATUS.md are present"
	@echo "  stress-test  Run 5-minute stress test (EDIT_STRESS_DURATION_SECS=300)"
	@echo "  ci-local     Full CI gate: fmt + clippy + test + smoke + bench"
	@echo ""
	@echo "Developer ergonomics (opt-in; see docs/dev-tmpfs.md):"
	@echo "  tmpfs-setup     Redirect target/ into /tmp/edit/<hash>/ to spare the SSD"
	@echo "  tmpfs-status    Show whether target/ is tmpfs-symlinked + disk usage"
	@echo "  tmpfs-teardown  Remove the symlink; pass WIPE=1 to also rm -rf the tmpfs subdir"
	@echo "  help         Show this help"
