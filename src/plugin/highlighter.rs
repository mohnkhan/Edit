//! Adapter that drives a Rhai highlighter plugin through the editor's existing
//! [`crate::highlight::Highlighter`] trait (Feature 008), so a plugin highlighter plugs into
//! `buffer.syntax` exactly like a built-in one and takes precedence for its extension.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use ratatui::style::{Color, Style};
use rhai::{Array, Engine};

use crate::highlight::{Highlighter, Span};
use crate::plugin::api::HostState;
use crate::plugin::sandbox::{arm_deadline, Deadline};
use crate::plugin::types::Permission;

/// A loaded Rhai highlighter bound to one file extension's content.
pub struct PluginHighlighter {
    pub plugin_id: String,
    pub ext: String,
    pub engine: Arc<Engine>,
    pub ast: Arc<rhai::AST>,
    pub deadline: Deadline,
    pub host: HostState,
    pub perms: Vec<Permission>,
    pub disabled: Arc<AtomicBool>,
    pub fs_violations: Arc<AtomicU32>,
    /// Theme colours indexed by [`crate::plugin::types::TokenKind::index`].
    pub colors: [Color; 7],
}

impl Highlighter for PluginHighlighter {
    fn highlight(&self, line: &str) -> Vec<Span> {
        if self.disabled.load(Ordering::Relaxed) {
            return Vec::new();
        }

        // Arm sandbox state for this call.
        arm_deadline(&self.deadline);
        if let Ok(mut h) = self.host.lock() {
            h.current_perms = self.perms.clone();
            h.current_plugin_id = self.plugin_id.clone();
            h.fs_violation = Some(self.fs_violations.clone());
        }

        let engine = self.engine.clone();
        let ast = self.ast.clone();
        let line_owned = line.to_string();
        let ext_owned = self.ext.clone();
        let call = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut scope = rhai::Scope::new();
            engine.call_fn::<Array>(&mut scope, &ast, "highlight", (line_owned, ext_owned))
        }));

        let array = match call {
            Ok(Ok(arr)) => arr,
            Ok(Err(e)) => {
                log::warn!(
                    "[plugin {}] highlight error; disabling for session: {e}",
                    self.plugin_id
                );
                self.disabled.store(true, Ordering::Relaxed);
                return Vec::new();
            }
            Err(_) => {
                log::error!(
                    "[plugin {}] highlight panicked; disabling for session",
                    self.plugin_id
                );
                self.disabled.store(true, Ordering::Relaxed);
                return Vec::new();
            }
        };

        match super::validate_tokens(array, line) {
            Some(tokens) => tokens
                .into_iter()
                .map(|t| Span {
                    start: t.byte_start as usize,
                    end: t.byte_end as usize,
                    style: Style::default().fg(self.colors[t.kind.index()]),
                })
                .collect(),
            // Invalid token array: discard, but keep the plugin enabled (per contract).
            None => Vec::new(),
        }
    }

    fn name(&self) -> &'static str {
        "plugin"
    }
}

/// Wrap a `PluginHighlighter` in a checked-out clone usable as a boxed trait object.
#[allow(clippy::too_many_arguments)]
pub fn make_highlighter(
    plugin_id: String,
    ext: String,
    engine: Arc<Engine>,
    ast: Arc<rhai::AST>,
    deadline: Deadline,
    host: HostState,
    perms: Vec<Permission>,
    disabled: Arc<AtomicBool>,
    fs_violations: Arc<AtomicU32>,
    colors: [Color; 7],
) -> Box<dyn Highlighter> {
    Box::new(PluginHighlighter {
        plugin_id,
        ext,
        engine,
        ast,
        deadline,
        host,
        perms,
        disabled,
        fs_violations,
        colors,
    })
}
