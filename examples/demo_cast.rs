//! Feature 035: generate an asciicast (v2) of a scripted `edit` session, used to
//! produce the README demo GIF (`make demo-gif` → `agg`).
//!
//! It drives the editor through the public API (`App::new`, `handle_action`,
//! `Ui::render`) and serializes each scene's rendered cell grid to a full-screen
//! ANSI repaint — fully deterministic, no PTY or key-delivery dependence. Writes
//! the cast to stdout.

use std::io::Write;

use ratatui::{backend::TestBackend, buffer::Buffer as TuiBuffer, style::Color, Terminal};

use edit::app::App;
use edit::config::Config;
use edit::encoding::EncodingId;
use edit::input::keymap::Action;
use edit::ui::Ui;

const W: u16 = 80;
const H: u16 = 24;

/// The classic CGA/DOS 16-color palette (so the GIF looks authentically DOS
/// regardless of the GIF renderer's own ANSI theme — we emit true-color).
fn dos_rgb(c: Color) -> Option<(u8, u8, u8)> {
    Some(match c {
        Color::Black => (0x00, 0x00, 0x00),
        Color::Red => (0xAA, 0x00, 0x00),
        Color::Green => (0x00, 0xAA, 0x00),
        Color::Yellow => (0xAA, 0x55, 0x00), // DOS "brown" (dark yellow)
        Color::Blue => (0x00, 0x00, 0xAA),   // the signature EDIT.COM blue
        Color::Magenta => (0xAA, 0x00, 0xAA),
        Color::Cyan => (0x00, 0xAA, 0xAA),
        Color::Gray => (0xAA, 0xAA, 0xAA),
        Color::DarkGray => (0x55, 0x55, 0x55),
        Color::LightRed => (0xFF, 0x55, 0x55),
        Color::LightGreen => (0x55, 0xFF, 0x55),
        Color::LightYellow => (0xFF, 0xFF, 0x55),
        Color::LightBlue => (0x55, 0x55, 0xFF),
        Color::LightMagenta => (0xFF, 0x55, 0xFF),
        Color::LightCyan => (0x55, 0xFF, 0xFF),
        Color::White => (0xFF, 0xFF, 0xFF),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(_) | Color::Reset => return None,
    })
}

/// Append the SGR parameters for a foreground/background color, as true-color so
/// the rendered GIF matches the DOS palette exactly.
fn sgr_color(out: &mut String, c: Color, bg: bool) {
    use std::fmt::Write as _;
    let lead = if bg { 48 } else { 38 };
    if let Some((r, g, b)) = dos_rgb(c) {
        let _ = write!(out, "{lead};2;{r};{g};{b};");
    } else {
        // Reset (handled by the caller's fallbacks) → default fg/bg.
        let _ = write!(out, "{};", if bg { 49 } else { 39 });
    }
}

/// Serialize a rendered cell grid to a full-screen ANSI repaint (home + styled
/// rows). Default fg/bg fall back to white-on-blue so the classic look reads even
/// on terminals without our theme.
fn frame_ansi(buf: &TuiBuffer) -> String {
    use ratatui::style::Modifier;
    let mut out = String::with_capacity((W as usize + 16) * H as usize);
    out.push_str("\u{1b}[H"); // cursor home
    for y in 0..H {
        out.push_str("\u{1b}[0m");
        for x in 0..W {
            let cell = buf.get(x, y);
            let mut sgr = String::from("\u{1b}[0m\u{1b}[");
            if cell.modifier.contains(Modifier::BOLD) {
                sgr.push_str("1;");
            }
            if cell.modifier.contains(Modifier::DIM) {
                sgr.push_str("2;");
            }
            if cell.modifier.contains(Modifier::REVERSED) {
                sgr.push_str("7;");
            }
            if cell.modifier.contains(Modifier::UNDERLINED) {
                sgr.push_str("4;");
            }
            let fg = if cell.fg == Color::Reset {
                Color::White
            } else {
                cell.fg
            };
            let bg = if cell.bg == Color::Reset {
                Color::Blue
            } else {
                cell.bg
            };
            sgr_color(&mut sgr, fg, false);
            sgr_color(&mut sgr, bg, true);
            // Replace a possible trailing ';' with 'm'.
            if sgr.ends_with(';') {
                sgr.pop();
            }
            sgr.push('m');
            out.push_str(&sgr);
            out.push_str(cell.symbol());
        }
        out.push_str("\u{1b}[0m");
        if y + 1 < H {
            out.push_str("\r\n");
        }
    }
    out
}

fn type_str(app: &mut App, s: &str) {
    for ch in s.chars() {
        if ch == '\n' {
            app.handle_action(Action::InsertNewline).unwrap();
        } else {
            app.handle_action(Action::InsertChar(ch)).unwrap();
        }
    }
}

fn main() {
    let mut app = App::new(Config::default(), vec![], EncodingId::Utf8, None, None);
    app.terminal_size = (W, H);
    let mut term = Terminal::new(TestBackend::new(W, H)).unwrap();

    // A scene = (mutate app, dwell seconds). We emit one cast event per scene.
    let mut events: Vec<(f64, String)> = Vec::new();
    let mut t = 0.0f64;
    let scene = |app: &mut App,
                 term: &mut Terminal<TestBackend>,
                 dwell: f64,
                 events: &mut Vec<(f64, String)>,
                 t: &mut f64| {
        term.draw(|f| Ui::render(f, app)).unwrap();
        let ansi = frame_ansi(term.backend().buffer());
        events.push((*t, ansi));
        *t += dwell;
    };

    // ── Scene 1: welcome ──────────────────────────────────────────────────────
    type_str(&mut app, "Welcome to edit — the DOS editor, reborn.\n\n");
    scene(&mut app, &mut term, 1.6, &mut events, &mut t);

    // ── Scene 2: UTF-8 everywhere (glyphs the GIF font is sure to have) ────────
    type_str(
        &mut app,
        "UTF-8 native: café, naïve, façade, €, ©, → — first-class.\n",
    );
    scene(&mut app, &mut term, 1.8, &mut events, &mut t);

    // ── Scene 3: word-wise selection (Ctrl+Shift+Right) ───────────────────────
    app.handle_action(Action::MoveDocStart).unwrap();
    app.handle_action(Action::SelectWordRight).unwrap();
    app.handle_action(Action::SelectWordRight).unwrap();
    scene(&mut app, &mut term, 1.6, &mut events, &mut t);

    // ── Scene 4: pull-down menu (Search) ──────────────────────────────────────
    app.handle_action(Action::MoveDocEnd).unwrap();
    app.handle_action(Action::MenuSearch).unwrap();
    scene(&mut app, &mut term, 1.8, &mut events, &mut t);
    app.handle_action(Action::MenuClose).unwrap();

    // ── Scene 5: Find dialog ──────────────────────────────────────────────────
    app.handle_action(Action::Find).unwrap();
    type_str(&mut app, "café");
    scene(&mut app, &mut term, 1.8, &mut events, &mut t);
    app.handle_action(Action::MenuClose).unwrap();

    // ── Scene 6: multiple buffers → tab bar ───────────────────────────────────
    let mut second = edit::buffer::Buffer::new_empty();
    second.path = Some(std::path::PathBuf::from("notes.md"));
    app.buffers.push(second);
    app.active_idx = 1;
    type_str(
        &mut app,
        "# notes.md\n\nA second buffer — see the tab bar above.\n",
    );
    scene(&mut app, &mut term, 2.0, &mut events, &mut t);

    // ── Scene 7: Help overlay ─────────────────────────────────────────────────
    app.handle_action(Action::Help).unwrap();
    scene(&mut app, &mut term, 2.2, &mut events, &mut t);
    app.handle_action(Action::MenuClose).unwrap();

    // Final dwell.
    scene(&mut app, &mut term, 1.2, &mut events, &mut t);

    // ── Emit asciicast v2 ─────────────────────────────────────────────────────
    let stdout = std::io::stdout();
    let mut o = stdout.lock();
    writeln!(
        o,
        "{{\"version\":2,\"width\":{W},\"height\":{H},\"title\":\"edit — DOS editor for the modern terminal\"}}"
    )
    .unwrap();
    // Hide the cursor for the whole cast.
    writeln!(o, "[0.0, \"o\", \"\\u001b[?25l\\u001b[2J\"]").unwrap();
    for (ts, ansi) in &events {
        let payload = json_escape(ansi);
        writeln!(o, "[{ts:.2}, \"o\", \"{payload}\"]").unwrap();
    }
}

/// Minimal JSON string escaping for the cast payload.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{1b}' => out.push_str("\\u001b"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}
