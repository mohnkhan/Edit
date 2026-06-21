# Quickstart: Per-Pane Wrap Cache
## Validate
```bash
make check     # render tests (split both-wrapped; single unchanged) + fuzz
make ci-local
```
## Manual
Open two files with long lines, enable split (View ▸ Split), turn soft-wrap on for each tab → BOTH panes
wrap to their half width with correct gutters/scrollbars; switching the active tab keeps both correct.
Single-pane view looks exactly as before.
## Refs: research.md, data-model.md, contracts/internal-api.md
