# Quickstart: Recent-Files List
## Validate
```bash
make check    # recent-store unit tests + menu injection + persistence
make ci-local
```
## Manual
Open a couple of files (File ▸ Open). Open the File menu → recent files listed (most-recent first);
choose one → it reopens. Quit and relaunch → the list persists. `recent_files_limit` in config caps it.
## Refs: research.md, data-model.md, contracts/internal-api.md
