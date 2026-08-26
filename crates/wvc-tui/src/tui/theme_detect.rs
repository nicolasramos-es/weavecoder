//! Terminal light/dark theme resolution.
//!
//! Resolves the theme mode once per process, before the TUI enters raw mode:
//!
//! 1. `WVC_THEME=dark|light` env override.
//! 2. `display.theme` config: "dark", "light", or "auto"/empty.
//! 3. Default: **dark**.
//!
//! Product decision: the TUI is always dark (black background, white / orange
//! text, opencode-style). The login/onboarding cards use fixed dark
//! backgrounds that are illegible on a light theme, and light text on a light
//! background is unreadable. So we never flip to a light palette from the
//! terminal's reported background — the default and the `auto` fallback are
//! both DARK. A user who really wants a light terminal opts in explicitly with
//! `WVC_THEME=light` or `display.theme = "light"`.
//!
//! The result is stored in `wvc_tui_style::theme_mode` where the renderer
//! adapts colors at frame time.

use std::sync::{Mutex, OnceLock};
use wvc_tui_style::ThemeMode;

static DETECTED: OnceLock<ThemeMode> = OnceLock::new();

/// Resolve the theme on a background thread so startup doesn't block on it.
/// Since resolution is now instant (no terminal query), this is effectively a
/// no-op wrapper kept for call-site compatibility.
static PREWARM: Mutex<Option<std::thread::JoinHandle<ThemeMode>>> = Mutex::new(None);

/// Start resolving the theme mode on a background thread. Idempotent, and a
/// no-op once the mode is already resolved.
pub fn prewarm_theme_mode() {
    if DETECTED.get().is_some() {
        return;
    }
    let Ok(mut slot) = PREWARM.lock() else {
        return;
    };
    if slot.is_some() {
        return;
    }
    if let Ok(handle) = std::thread::Builder::new()
        .name("wvc-theme-detect".to_string())
        .spawn(resolve_theme_mode)
    {
        *slot = Some(handle);
    }
}

/// Join a prewarm started by [`prewarm_theme_mode`], if any.
fn take_prewarmed_theme_mode() -> Option<ThemeMode> {
    let handle = PREWARM.lock().ok()?.take()?;
    handle.join().ok()
}

/// Resolve and install the global theme mode. Idempotent; the first call does
/// the (instant) resolution and later calls are free. Must be called before
/// entering raw mode / the alternate screen.
pub fn init_theme_mode() -> ThemeMode {
    let mode = match take_prewarmed_theme_mode() {
        Some(prewarmed) => *DETECTED.get_or_init(|| prewarmed),
        None => *DETECTED.get_or_init(resolve_theme_mode),
    };
    wvc_tui_style::set_theme_mode(mode);
    init_palette();
    mode
}

/// Resolve the theme while resuming an already-active TUI after an `exec` handoff.
///
/// The inherited terminal is already in raw mode. Prefer the theme captured by
/// the previous process and otherwise resolve configuration (always dark unless
/// overridden) without querying the terminal.
pub fn init_theme_mode_for_resume(inherited_theme: Option<&str>) -> ThemeMode {
    let inherited_theme = inherited_theme.and_then(|value| match value {
        "dark" => Some(ThemeMode::Dark),
        "light" => Some(ThemeMode::Light),
        _ => None,
    });
    let prewarmed = take_prewarmed_theme_mode();
    let mode = *DETECTED.get_or_init(|| {
        inherited_theme
            .or(prewarmed)
            .unwrap_or_else(resolve_theme_mode_without_terminal_query)
    });
    wvc_tui_style::set_theme_mode(mode);
    init_palette();
    mode
}

/// Install the user's configured color palette from `[display.colors]`.
///
/// Invalid entries are logged and skipped rather than failing the palette, so
/// one typo can never leave the TUI unstyled. Safe to call repeatedly; the TUI
/// calls it again after `/colors` edits so changes apply without a restart.
pub fn init_palette() {
    let configured = &crate::config::config().display.colors;
    let (palette, errors) = wvc_tui_style::Palette::from_pairs(
        configured
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    );
    for error in errors {
        crate::logging::warn(&format!("display.colors: {error}"));
    }
    wvc_tui_style::set_palette(palette);
}

pub fn current_theme_label() -> &'static str {
    match wvc_tui_style::theme_mode() {
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
    }
}

fn resolve_theme_mode() -> ThemeMode {
    resolve_configured_theme()
}

fn resolve_theme_mode_without_terminal_query() -> ThemeMode {
    resolve_configured_theme()
}

fn resolve_configured_theme() -> ThemeMode {
    let configured = std::env::var("WVC_THEME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| crate::config::config().display.theme.clone());

    match configured.trim().to_ascii_lowercase().as_str() {
        "dark" => ThemeMode::Dark,
        "light" => ThemeMode::Light,
        // Default to dark (never flip to light from the terminal's reported
        // background — see module doc).
        "" | "auto" => ThemeMode::Dark,
        other => {
            crate::logging::info(&format!(
                "Unknown theme '{other}' (expected auto/dark/light); using dark theme"
            ));
            ThemeMode::Dark
        }
    }
}
