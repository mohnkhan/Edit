# Quickstart: Restore Scroll/Selection/Encoding
## Validate
```bash
make check    # round-trip + legacy + clamp tests + existing 003/045 session tests
make ci-local # fmt -> clippy -D warnings -> test -> smoke -> perf-check
```
Expected green; legacy session files load with defaults.
## Manual
Open a file, scroll down, select a range, Save-As-Encoding to UTF-16, quit (saves session), relaunch with
no args, accept restore → reopens scrolled, selection active, decoded UTF-16. Old session.toml still loads.
## Refs: research.md, data-model.md, contracts/internal-api.md
