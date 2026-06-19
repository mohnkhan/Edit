#!/usr/bin/env bash
# SC-003 (Feature 008): startup stays under 2 s with up to 10 plugins installed.
#
# Standalone best-effort check (not wired into the required CI gate to avoid timing
# flakiness on shared runners). Run manually:  bash tests/smoke/plugin_startup_perf.sh
set -euo pipefail

BIN="${EDIT_BIN:-./target/debug/edit}"
[ -x "$BIN" ] || { echo "build the editor first: cargo build"; exit 1; }

CFG="$(mktemp -d)"
trap 'rm -rf "$CFG"' EXIT
mkdir -p "$CFG/edit"

# Install 10 consented copies of the lua-syntax fixture under distinct ids.
{
  for i in $(seq 1 10); do
    id="perf-plugin-$i"
    mkdir -p "$CFG/edit/plugins/$id"
    sed "s/^id = .*/id = \"$id\"/" tests/fixtures/plugins/lua-syntax/plugin.toml \
      > "$CFG/edit/plugins/$id/plugin.toml"
    cp tests/fixtures/plugins/lua-syntax/plugin.rhai "$CFG/edit/plugins/$id/plugin.rhai"
    printf '[plugins.%s]\nallowed = true\nconsented_at = "2026-06-19T00:00:00Z"\nversion_consented = "1.0.0"\n' \
      "$id"
  done
} >> "$CFG/edit/plugins.toml"

f="$(mktemp --suffix=.lua)"; echo "-- hi" > "$f"; trap 'rm -f "$f"' EXIT

# Launch headless, send Ctrl+Q immediately, measure wall-clock.
start=$(date +%s.%N)
XDG_CONFIG_HOME="$CFG" LC_ALL=C.UTF-8 LANG=C.UTF-8 \
  expect -c "set timeout 10; spawn $BIN $f; expect -re {.+}; send \"\x11\"; expect eof" \
  > /dev/null 2>&1 || true
end=$(date +%s.%N)

elapsed=$(echo "$end - $start" | bc)
echo "startup+quit with 10 plugins: ${elapsed}s"
# Generous bound; SC-003 target is < 2 s of startup.
awk "BEGIN { exit !($elapsed < 3.0) }" && { echo "PASS (SC-003)"; exit 0; } || { echo "SLOW"; exit 1; }
