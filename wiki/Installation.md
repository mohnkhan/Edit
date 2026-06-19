# Installation

`edit` builds to a single, minimally dependent binary. This page covers prerequisites, building from
source, producing OS packages, supported targets, and running the result.

## Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust toolchain | **stable 1.74.0 or newer** | MSRV is 1.74 (required for `ratatui` 0.26 and `clap` 4) |
| `cargo` | ships with Rust | the build driver |
| A terminal | any | one that supports the `crossterm` protocol; for mouse support it must report mouse events |

Optional, for specific workflows:

| Tool | Needed for |
|---|---|
| `cargo-deb` | `make package-deb` |
| `rpmbuild` | `make package-rpm` |
| Rust **nightly** + `x86_64-unknown-linux-musl` target | `make static` (musl static binary) |
| `expect` (+ `tmux`) | `make smoke` (smoke tests) |

Install Rust with [rustup](https://rustup.rs) if you don't have it:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building from source

Clone the repository, then use the provided `Makefile` (which wraps `cargo`).

```sh
# Debug build → target/debug/edit
make build      # or: cargo build

# Optimized release build (LTO, stripped, -O3) → target/release/edit
make release    # or: cargo build --release
```

### Static musl build

For a fully static binary with no glibc dependency — ideal for the MyOS image or for dropping onto
any Linux host — build the `release-static` profile against the musl target. This requires the musl
target and a nightly toolchain:

```sh
rustup target add x86_64-unknown-linux-musl

make static
# → target/x86_64-unknown-linux-musl/release-static/edit
```

## Packaging

```sh
# Debian / Ubuntu .deb (requires cargo-deb)
make package-deb

# RPM (requires rpmbuild; builds release first, then packaging/edit.spec)
make package-rpm
```

The Debian package installs the binary to `/usr/bin/edit` and the man page to
`/usr/share/man/man1/edit.1`.

## Supported targets

| Target | Toolchain | Profile | Notes |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Primary development target |
| `aarch64-unknown-linux-gnu` | stable 1.74.0+ | debug, release | Cross-compiled (e.g. via `cross`) |
| `x86_64-unknown-linux-musl` | nightly | release-static | Static binary, no glibc dependency |

There is **no DOS/DPMI runtime dependency** and **no X11/Wayland dependency** — `edit` runs in any
Linux terminal.

## Running

```sh
# Launch with a blank buffer
./target/release/edit

# Open one or more files
./target/release/edit notes.txt README.md

# Print version / help
./target/release/edit --version
./target/release/edit --help
```

The full set of command-line options is documented on the [CLI Reference](CLI-Reference.md) page.
For a guided first run, see [Getting Started](Getting-Started.md).
