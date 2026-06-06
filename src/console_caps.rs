//! `ConsoleCapabilities` — detected terminal capability flags.
//!
//! All detection is environment-only (no blocking DECRQM probes),
//! so it is safe on WASM targets and never blocks.
//!
//! # Design
//!
//! The core helper [`ConsoleCapabilities::from_env_parts`] is a pure function
//! that takes pre-read env-var slices, making it fully testable without
//! mutating the process environment.  The public constructor
//! [`ConsoleCapabilities::from_env`] reads the actual environment and
//! delegates to it — exactly mirroring the `detect_color_system_from` pattern
//! already used in `console.rs`.

use crate::color::ColorSystem;
use crate::console::detect_color_system_from;

// ---------------------------------------------------------------------------
// ConsoleCapabilities
// ---------------------------------------------------------------------------

/// Detected terminal capability flags, derived from environment variables.
///
/// Constructed by `Console::new()` / `Console::from_builder()` and accessible
/// via [`Console::capabilities`].
///
/// All probing is **environment-only**: no blocking stdin reads, no DECRQM
/// queries.  Those are deferred to a future `terminal-query` feature flag.
///
/// # Fields
///
/// * `color_system` — The resolved `ColorSystem` (same value as
///   `Console::color_system()`; kept here for one-stop capability access).
/// * `is_terminal` — Whether stdout was detected as a terminal.
/// * `truecolor` — `COLORTERM` contains `truecolor` or `24bit`.
/// * `synchronized_output` — Always `true` (default); the CSI ?2026 sequences
///   are harmless no-ops on terminals that don't support DEC Mode 2026.
///   Reserved for future opt-out via env var or a later probing feature.
/// * `unicode_version` — `UNICODE_VERSION` env var parsed as `u32`, if set.
/// * `kitty` — Terminal supports the Kitty graphics protocol (APC `\x1b_G`).
///   Detected from `TERM=xterm-kitty`, `KITTY_WINDOW_ID` being set,
///   `TERM_PROGRAM=WezTerm`, or `TERM_PROGRAM=ghostty`.
/// * `sixel` — Terminal supports Sixel graphics. Currently always `false`
///   (conservative default; a future `terminal-query` feature may probe it).
/// * `iterm` — Terminal supports the iTerm2 inline image protocol.
///   Detected from `TERM_PROGRAM=iTerm.app`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCapabilities {
    /// The active color system.
    pub color_system: ColorSystem,
    /// Whether stdout appears to be an interactive terminal.
    pub is_terminal: bool,
    /// Whether the terminal advertises 24-bit / truecolor support.
    pub truecolor: bool,
    /// Synchronized-output (DEC Mode 2026) support flag.
    ///
    /// Defaults to `true` because the CSI sequences are harmless no-ops on
    /// unsupported terminals.  Future releases may probe via DECRQM and flip
    /// this to `false` when the terminal reports unsupported.
    pub synchronized_output: bool,
    /// Unicode version hint from the `UNICODE_VERSION` environment variable,
    /// or `None` when not set / not parseable.
    pub unicode_version: Option<u32>,
    /// Whether the terminal supports the Kitty graphics protocol.
    ///
    /// Detected from `TERM=xterm-kitty`, `KITTY_WINDOW_ID` env var,
    /// `TERM_PROGRAM=WezTerm`, or `TERM_PROGRAM=ghostty`.
    pub kitty: bool,
    /// Whether the terminal supports Sixel graphics.
    ///
    /// Currently always `false` (conservative default). A future
    /// `terminal-query` feature may use DECRQM to probe this.
    pub sixel: bool,
    /// Whether the terminal supports the iTerm2 inline image protocol.
    ///
    /// Detected from `TERM_PROGRAM=iTerm.app`.
    pub iterm: bool,
}

impl ConsoleCapabilities {
    /// Build capabilities from pre-read environment-variable values.
    ///
    /// This is the **testable pure helper**: callers pass in the relevant
    /// env-var strings so tests can exercise every code path without touching
    /// the process environment.
    ///
    /// # Arguments
    ///
    /// * `colorterm` — Value of `COLORTERM` (e.g. `"truecolor"`, `"24bit"`, or `None`).
    /// * `term` — Value of `TERM` (e.g. `"xterm-256color"`, or `None`).
    /// * `is_terminal` — Whether the output stream was detected as a terminal.
    /// * `unicode_version_str` — Value of `UNICODE_VERSION`, if set.
    /// * `kitty_window_id` — Value of `KITTY_WINDOW_ID`, if set (any value → kitty=true).
    /// * `term_program` — Value of `TERM_PROGRAM`, if set (e.g. `"WezTerm"`, `"iTerm.app"`).
    pub fn from_env_parts(
        colorterm: Option<&str>,
        term: Option<&str>,
        is_terminal: bool,
        unicode_version_str: Option<&str>,
        kitty_window_id: Option<&str>,
        term_program: Option<&str>,
    ) -> Self {
        let truecolor = colorterm.is_some_and(|ct| {
            let ct_lower = ct.to_lowercase();
            ct_lower.contains("truecolor") || ct_lower.contains("24bit")
        });

        let color_system = detect_color_system_from(colorterm, term);

        let unicode_version = unicode_version_str.and_then(|s| s.parse::<u32>().ok());

        // Kitty graphics protocol detection.
        // Signals: TERM=xterm-kitty, KITTY_WINDOW_ID set (any value),
        //          TERM_PROGRAM=WezTerm, TERM_PROGRAM=ghostty.
        let kitty = term.is_some_and(|t| t == "xterm-kitty")
            || kitty_window_id.is_some()
            || term_program.is_some_and(|tp| {
                let tp_lower = tp.to_lowercase();
                tp_lower == "wezterm" || tp_lower == "ghostty"
            });

        // iTerm2 inline image protocol.
        let iterm = term_program.is_some_and(|tp| tp == "iTerm.app");

        // Sixel: conservative default false. No reliable env-only signal.
        let sixel = false;

        ConsoleCapabilities {
            color_system,
            is_terminal,
            truecolor,
            synchronized_output: true,
            unicode_version,
            kitty,
            sixel,
            iterm,
        }
    }

    /// Build capabilities by reading the real process environment.
    ///
    /// Reads `COLORTERM`, `TERM`, `UNICODE_VERSION`, `KITTY_WINDOW_ID`,
    /// `TERM_PROGRAM`, and detects whether stdout is a terminal.
    /// This is the constructor called by `Console::from_builder`.
    pub fn from_env(is_terminal: bool) -> Self {
        let colorterm = std::env::var("COLORTERM").ok();
        let term = std::env::var("TERM").ok();
        let unicode_version_str = std::env::var("UNICODE_VERSION").ok();
        let kitty_window_id = std::env::var("KITTY_WINDOW_ID").ok();
        let term_program = std::env::var("TERM_PROGRAM").ok();

        Self::from_env_parts(
            colorterm.as_deref(),
            term.as_deref(),
            is_terminal,
            unicode_version_str.as_deref(),
            kitty_window_id.as_deref(),
            term_program.as_deref(),
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorSystem;

    /// COLORTERM=truecolor → truecolor flag is true, color_system is TrueColor.
    #[test]
    fn caps_truecolor_flag_when_colorterm_truecolor() {
        let caps =
            ConsoleCapabilities::from_env_parts(Some("truecolor"), None, true, None, None, None);
        assert!(caps.truecolor, "truecolor flag should be true");
        assert_eq!(caps.color_system, ColorSystem::TrueColor);
        assert!(caps.is_terminal);
    }

    /// COLORTERM=24bit → truecolor flag is true.
    #[test]
    fn caps_truecolor_flag_when_colorterm_24bit() {
        let caps = ConsoleCapabilities::from_env_parts(Some("24bit"), None, true, None, None, None);
        assert!(caps.truecolor, "24bit should set truecolor flag");
    }

    /// No COLORTERM, standard TERM → truecolor is false.
    #[test]
    fn caps_no_truecolor_when_no_colorterm() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-256color"),
            true,
            None,
            None,
            None,
        );
        assert!(!caps.truecolor, "no COLORTERM → truecolor should be false");
        assert_eq!(caps.color_system, ColorSystem::EightBit);
    }

    /// `synchronized_output` defaults to `true` regardless of env.
    #[test]
    fn caps_synchronized_output_defaults_true() {
        let caps = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None);
        assert!(
            caps.synchronized_output,
            "synchronized_output should default to true"
        );
    }

    /// `UNICODE_VERSION=15` → `unicode_version` is `Some(15)`.
    #[test]
    fn caps_unicode_version_parsed_from_env_parts() {
        let caps = ConsoleCapabilities::from_env_parts(None, None, false, Some("15"), None, None);
        assert_eq!(caps.unicode_version, Some(15));
    }

    /// Non-numeric `UNICODE_VERSION` → `None`.
    #[test]
    fn caps_unicode_version_none_on_non_numeric() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, None, false, Some("fifteen"), None, None);
        assert_eq!(caps.unicode_version, None);
    }

    /// `unicode_version` is `None` when env var is absent.
    #[test]
    fn caps_unicode_version_none_when_absent() {
        let caps = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None);
        assert_eq!(caps.unicode_version, None);
    }

    /// is_terminal propagated correctly.
    #[test]
    fn caps_is_terminal_propagated() {
        let tty = ConsoleCapabilities::from_env_parts(None, None, true, None, None, None);
        assert!(tty.is_terminal);
        let piped = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None);
        assert!(!piped.is_terminal);
    }

    /// from_env() is callable without panicking (smoke test).
    #[test]
    fn caps_from_env_smoke() {
        // Just ensure it doesn't panic; we don't control the env here.
        let _caps = ConsoleCapabilities::from_env(false);
    }
}
