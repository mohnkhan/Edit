// Library entry point — exposes internal modules for integration tests and
// potential future use as a library crate.
//
// Only the modules needed for integration tests are re-exported here.
// The binary entry point remains src/main.rs.

#![allow(dead_code)]

pub mod buffer;
pub mod config;
pub mod diagnostics;
pub mod encoding;
pub mod highlight;
pub mod input;
pub mod search;
pub mod security;
pub mod session;
pub mod ui;

// `app` depends on the full TUI stack; expose it as well so integration tests
// can drive App directly if needed in future.
pub mod app;
pub mod watcher;
