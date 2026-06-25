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
/// * `sixel` — Terminal supports Sixel graphics. Detected from a conservative
///   env-only allowlist of known sixel-capable terminals (e.g. `TERM`
///   containing `foot`/`contour`/`mlterm`/`yaft`, or `TERM_PROGRAM` of
///   `contour`/`foot`/`WezTerm`/`mlterm`).
/// * `iterm` — Terminal supports the iTerm2 inline image protocol.
///   Detected from `TERM_PROGRAM=iTerm.app`.
///
/// Any of `kitty`/`sixel`/`iterm` can be globally forced via the
/// `GILT_IMAGE_PROTOCOL` environment variable (`kitty`/`iterm`/`sixel`/`halfblock`).
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
    /// Detected from a conservative env-only allowlist of terminals known to
    /// implement Sixel: `TERM` containing `foot`/`contour`/`mlterm`/`yaft`, or
    /// `TERM_PROGRAM` of `contour`/`foot`/`WezTerm`/`mlterm`. A future
    /// `terminal-query` feature may use DECRQM to widen detection.
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
    /// * `image_protocol_override` — Value of `GILT_IMAGE_PROTOCOL`, if set. A
    ///   recognized value (`kitty`/`iterm`/`sixel`/`halfblock`, case-insensitive,
    ///   surrounding whitespace ignored) forces exactly that protocol's flags,
    ///   overriding all heuristics; `halfblock` disables every inline-image
    ///   protocol. Unrecognized / absent values leave detection intact.
    pub fn from_env_parts(
        colorterm: Option<&str>,
        term: Option<&str>,
        is_terminal: bool,
        unicode_version_str: Option<&str>,
        kitty_window_id: Option<&str>,
        term_program: Option<&str>,
        image_protocol_override: Option<&str>,
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

        // Sixel graphics. Conservative env-only allowlist of terminals KNOWN to
        // implement Sixel — two signals, both case-insensitive:
        //   * TERM substring:        foot, contour, mlterm, yaft
        //   * TERM_PROGRAM (exact):  contour, foot, WezTerm, mlterm
        // (A future `terminal-query` feature may DECRQM-probe to widen this.)
        let sixel = term.is_some_and(|t| {
            let t_lower = t.to_lowercase();
            t_lower.contains("foot")
                || t_lower.contains("contour")
                || t_lower.contains("mlterm")
                || t_lower.contains("yaft")
        }) || term_program.is_some_and(|tp| {
            let tp_lower = tp.to_lowercase();
            tp_lower == "contour"
                || tp_lower == "foot"
                || tp_lower == "wezterm"
                || tp_lower == "mlterm"
        });

        // `GILT_IMAGE_PROTOCOL` global override. A recognized value forces
        // exactly one protocol's flag set, overriding every heuristic above;
        // `halfblock` disables all inline-image protocols (forcing the
        // half-block fallback renderer). Matching is case-insensitive and
        // ignores surrounding whitespace. Unrecognized / absent values leave
        // the detected flags untouched.
        let (kitty, iterm, sixel) = match image_protocol_override.map(str::trim) {
            Some(v) if v.eq_ignore_ascii_case("kitty") => (true, false, false),
            Some(v) if v.eq_ignore_ascii_case("iterm") => (false, true, false),
            Some(v) if v.eq_ignore_ascii_case("sixel") => (false, false, true),
            Some(v) if v.eq_ignore_ascii_case("halfblock") => (false, false, false),
            _ => (kitty, iterm, sixel),
        };

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
    /// `TERM_PROGRAM`, `GILT_IMAGE_PROTOCOL`, and detects whether stdout is a
    /// terminal. This is the constructor called by `Console::from_builder`.
    ///
    /// `GILT_IMAGE_PROTOCOL` lets users globally override inline-image protocol
    /// detection — set it to `kitty`, `iterm`, `sixel`, or `halfblock` to force
    /// that protocol regardless of the detected terminal.
    pub fn from_env(is_terminal: bool) -> Self {
        let colorterm = std::env::var("COLORTERM").ok();
        let term = std::env::var("TERM").ok();
        let unicode_version_str = std::env::var("UNICODE_VERSION").ok();
        let kitty_window_id = std::env::var("KITTY_WINDOW_ID").ok();
        let term_program = std::env::var("TERM_PROGRAM").ok();
        let image_protocol_override = std::env::var("GILT_IMAGE_PROTOCOL").ok();

        Self::from_env_parts(
            colorterm.as_deref(),
            term.as_deref(),
            is_terminal,
            unicode_version_str.as_deref(),
            kitty_window_id.as_deref(),
            term_program.as_deref(),
            image_protocol_override.as_deref(),
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
        let caps = ConsoleCapabilities::from_env_parts(
            Some("truecolor"),
            None,
            true,
            None,
            None,
            None,
            None,
        );
        assert!(caps.truecolor, "truecolor flag should be true");
        assert_eq!(caps.color_system, ColorSystem::TrueColor);
        assert!(caps.is_terminal);
    }

    /// COLORTERM=24bit → truecolor flag is true.
    #[test]
    fn caps_truecolor_flag_when_colorterm_24bit() {
        let caps =
            ConsoleCapabilities::from_env_parts(Some("24bit"), None, true, None, None, None, None);
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
            None,
        );
        assert!(!caps.truecolor, "no COLORTERM → truecolor should be false");
        assert_eq!(caps.color_system, ColorSystem::EightBit);
    }

    /// `synchronized_output` defaults to `true` regardless of env.
    #[test]
    fn caps_synchronized_output_defaults_true() {
        let caps = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None, None);
        assert!(
            caps.synchronized_output,
            "synchronized_output should default to true"
        );
    }

    /// `UNICODE_VERSION=15` → `unicode_version` is `Some(15)`.
    #[test]
    fn caps_unicode_version_parsed_from_env_parts() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, None, false, Some("15"), None, None, None);
        assert_eq!(caps.unicode_version, Some(15));
    }

    /// Non-numeric `UNICODE_VERSION` → `None`.
    #[test]
    fn caps_unicode_version_none_on_non_numeric() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            false,
            Some("fifteen"),
            None,
            None,
            None,
        );
        assert_eq!(caps.unicode_version, None);
    }

    /// `unicode_version` is `None` when env var is absent.
    #[test]
    fn caps_unicode_version_none_when_absent() {
        let caps = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None, None);
        assert_eq!(caps.unicode_version, None);
    }

    /// is_terminal propagated correctly.
    #[test]
    fn caps_is_terminal_propagated() {
        let tty = ConsoleCapabilities::from_env_parts(None, None, true, None, None, None, None);
        assert!(tty.is_terminal);
        let piped = ConsoleCapabilities::from_env_parts(None, None, false, None, None, None, None);
        assert!(!piped.is_terminal);
    }

    /// from_env() is callable without panicking (smoke test).
    #[test]
    fn caps_from_env_smoke() {
        // Just ensure it doesn't panic; we don't control the env here.
        let _caps = ConsoleCapabilities::from_env(false);
    }

    // -- Sixel env heuristics (allowlist of known sixel-capable terminals) ----

    /// `TERM` containing `foot` → sixel=true.
    #[test]
    fn caps_sixel_flag_from_foot_term() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, Some("foot"), true, None, None, None, None);
        assert!(caps.sixel, "TERM=foot should set sixel=true");
    }

    /// `TERM` containing `contour` → sixel=true.
    #[test]
    fn caps_sixel_flag_from_contour_term() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("contour-latest"),
            true,
            None,
            None,
            None,
            None,
        );
        assert!(caps.sixel, "TERM containing contour should set sixel=true");
    }

    /// `TERM` containing `mlterm` → sixel=true.
    #[test]
    fn caps_sixel_flag_from_mlterm_term() {
        let caps =
            ConsoleCapabilities::from_env_parts(None, Some("mlterm"), true, None, None, None, None);
        assert!(caps.sixel, "TERM=mlterm should set sixel=true");
    }

    /// `TERM` containing `yaft` → sixel=true.
    #[test]
    fn caps_sixel_flag_from_yaft_term() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("yaft-256color"),
            true,
            None,
            None,
            None,
            None,
        );
        assert!(caps.sixel, "TERM containing yaft should set sixel=true");
    }

    /// `TERM_PROGRAM=contour` → sixel=true.
    #[test]
    fn caps_sixel_flag_from_term_program_contour() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            Some("contour"),
            None,
        );
        assert!(caps.sixel, "TERM_PROGRAM=contour should set sixel=true");
    }

    /// `TERM_PROGRAM=WezTerm` → sixel=true (and kitty=true; flags are independent).
    #[test]
    fn caps_sixel_flag_from_term_program_wezterm() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            Some("WezTerm"),
            None,
        );
        assert!(caps.sixel, "WezTerm supports sixel");
        assert!(caps.kitty, "WezTerm also supports the kitty protocol");
    }

    /// Plain `xterm-256color` does NOT set sixel.
    #[test]
    fn caps_no_sixel_on_plain_term() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-256color"),
            true,
            None,
            None,
            None,
            None,
        );
        assert!(
            !caps.sixel,
            "plain xterm-256color should NOT set sixel=true"
        );
    }

    // -- GILT_IMAGE_PROTOCOL override ----------------------------------------

    /// Override `kitty` forces kitty=true and clears iterm/sixel, even when the
    /// terminal was detected as iTerm.
    #[test]
    fn caps_override_kitty_forces_kitty_only() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            Some("iTerm.app"),
            Some("kitty"),
        );
        assert!(caps.kitty, "override kitty → kitty=true");
        assert!(!caps.iterm, "override kitty → iterm=false");
        assert!(!caps.sixel, "override kitty → sixel=false");
    }

    /// Override `iterm` forces iterm=true and clears kitty/sixel, even when the
    /// terminal was detected as kitty.
    #[test]
    fn caps_override_iterm_forces_iterm_only() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-kitty"),
            true,
            None,
            None,
            None,
            Some("iterm"),
        );
        assert!(!caps.kitty, "override iterm → kitty=false");
        assert!(caps.iterm, "override iterm → iterm=true");
        assert!(!caps.sixel, "override iterm → sixel=false");
    }

    /// Override `sixel` forces sixel=true and clears kitty/iterm.
    #[test]
    fn caps_override_sixel_forces_sixel_only() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-kitty"),
            true,
            None,
            None,
            None,
            Some("sixel"),
        );
        assert!(!caps.kitty, "override sixel → kitty=false");
        assert!(!caps.iterm, "override sixel → iterm=false");
        assert!(caps.sixel, "override sixel → sixel=true");
    }

    /// Override `halfblock` disables every inline-image protocol.
    #[test]
    fn caps_override_halfblock_disables_all_protocols() {
        // WezTerm would otherwise be detected as kitty+sixel.
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            Some("WezTerm"),
            Some("halfblock"),
        );
        assert!(!caps.kitty, "override halfblock → kitty=false");
        assert!(!caps.iterm, "override halfblock → iterm=false");
        assert!(!caps.sixel, "override halfblock → sixel=false");
    }

    /// An unrecognized override value is ignored — detection is left intact.
    #[test]
    fn caps_override_unknown_value_ignored() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-kitty"),
            true,
            None,
            None,
            None,
            Some("bogus"),
        );
        assert!(caps.kitty, "unknown override must not disturb detection");
    }

    /// Absent override leaves detection intact.
    #[test]
    fn caps_override_absent_leaves_detection_intact() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            Some("xterm-kitty"),
            true,
            None,
            None,
            None,
            None,
        );
        assert!(caps.kitty, "absent override → detected kitty preserved");
    }

    /// Override matching is case-insensitive and trims surrounding whitespace.
    #[test]
    fn caps_override_is_case_insensitive_and_trimmed() {
        let caps = ConsoleCapabilities::from_env_parts(
            None,
            None,
            true,
            None,
            None,
            None,
            Some("  KITTY  "),
        );
        assert!(caps.kitty, "  KITTY  should force kitty=true");
        assert!(!caps.iterm);
        assert!(!caps.sixel);
    }
}
