# Makefile for Linux EDIT.COM Clone
# Targets: build release check smoke perf-check static package-deb package-rpm docs-gate ci-local help

BINARY     := edit
CARGO      := cargo
TARGET_DIR := target

.PHONY: build release check smoke perf-check static package-deb package-rpm docs-gate ci-local help

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
		expect "$$f" || exit 1; \
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
	@echo "  help         Show this help"
