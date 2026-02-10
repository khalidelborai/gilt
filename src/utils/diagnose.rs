//! Diagnostic tool for reporting terminal capabilities.
//!
//! Similar to Python Rich's diagnose.py, this module provides functionality
//! to inspect and report terminal capabilities for debugging purposes.
//!
//! # Example
//!
//! ```
//! use gilt::diagnose;
//!
//! // Print diagnostic report to console
//! diagnose::print_report();
//!
//! // Or get the report as a string
//! let report = diagnose::report();
//! ```

use std::collections::HashMap;
use std::env;

use crate::color::ColorSystem;
use crate::color_env::{detect_color_env, ColorEnvOverride};
use crate::console::Console;

/// Terminal information structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    /// Terminal type from $TERM environment variable.
    pub term: String,
    /// Terminal width in columns.
    pub width: usize,
    /// Terminal height in rows.
    pub height: usize,
    /// Detected terminal emulator, if known.
    pub emulator: Option<String>,
    /// Whether the output is an interactive terminal.
    pub is_terminal: bool,
}

impl TerminalInfo {
    /// Gather terminal information.
    pub fn detect() -> Self {
        let term = env::var("TERM").unwrap_or_default();
        let (width, height) = Console::detect_terminal_size();
        let emulator = detect_terminal_emulator();
        let is_terminal = is_terminal();

        Self {
            term,
            width,
            height,
            emulator,
            is_terminal,
        }
    }
}

/// Color support levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSupport {
    /// No color support.
    NoColor,
    /// Standard 16 colors.
    Standard,
    /// 256 colors (8-bit).
    Two56,
    /// TrueColor (24-bit).
    TrueColor,
}

impl ColorSupport {
    /// Detect color support from environment and terminal capabilities.
    pub fn detect() -> Self {
        match detect_color_env() {
            ColorEnvOverride::NoColor => ColorSupport::NoColor,
            ColorEnvOverride::ForceColor => ColorSupport::Two56,
            ColorEnvOverride::ForceColorTruecolor => ColorSupport::TrueColor,
            ColorEnvOverride::None => {
                // Check COLORTERM for truecolor support
                if let Ok(colorterm) = env::var("COLORTERM") {
                    let ct = colorterm.to_lowercase();
                    if ct.contains("truecolor") || ct.contains("24bit") {
                        return ColorSupport::TrueColor;
                    }
                }

                // Check TERM for color support hints
                if let Ok(term) = env::var("TERM") {
                    let term_lower = term.to_lowercase();
                    if term_lower.contains("256color") || term_lower.contains("256-color") {
                        return ColorSupport::Two56;
                    }
                    if term_lower.contains("truecolor") || term_lower.contains("24bit") {
                        return ColorSupport::TrueColor;
                    }
                }

                // Default to TrueColor for modern terminals
                ColorSupport::TrueColor
            }
        }
    }

    /// Get the name of the color support level.
    pub fn name(&self) -> &'static str {
        match self {
            ColorSupport::NoColor => "No Color",
            ColorSupport::Standard => "Standard (16 colors)",
            ColorSupport::Two56 => "256 colors",
            ColorSupport::TrueColor => "TrueColor (24-bit)",
        }
    }

    /// Get the corresponding ColorSystem, if any.
    pub fn color_system(&self) -> Option<ColorSystem> {
        match self {
            ColorSupport::NoColor => None,
            ColorSupport::Standard => Some(ColorSystem::Standard),
            ColorSupport::Two56 => Some(ColorSystem::EightBit),
            ColorSupport::TrueColor => Some(ColorSystem::TrueColor),
        }
    }
}

/// Unicode support information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeSupport {
    /// Whether UTF-8 encoding is supported.
    pub utf8_supported: bool,
    /// Detected Unicode version support level.
    pub version: String,
    /// Whether emoji rendering is supported.
    pub emoji_supported: bool,
    /// Whether box drawing characters are supported.
    pub box_drawing_supported: bool,
    /// Whether wide character support is available.
    pub wide_character_supported: bool,
}

impl UnicodeSupport {
    /// Detect Unicode capabilities.
    pub fn detect() -> Self {
        let utf8_supported = detect_utf8_support();
        let version = detect_unicode_version();
        let emoji_supported = detect_emoji_support();
        let box_drawing_supported = detect_box_drawing_support();
        let wide_character_supported = true; // Most modern terminals support this

        Self {
            utf8_supported,
            version,
            emoji_supported,
            box_drawing_supported,
            wide_character_supported,
        }
    }
}

/// Platform information structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    /// Operating system name.
    pub os: String,
    /// System architecture.
    pub arch: String,
    /// Rust version used to compile.
    pub rust_version: String,
    /// gilt library version.
    pub gilt_version: &'static str,
}

impl PlatformInfo {
    /// Gather platform information.
    pub fn detect() -> Self {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let rust_version = rustc_version();
        let gilt_version = env!("CARGO_PKG_VERSION");

        Self {
            os,
            arch,
            rust_version,
            gilt_version,
        }
    }
}

/// Complete diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    /// Terminal information.
    pub terminal: TerminalInfo,
    /// Relevant environment variables.
    pub environment: HashMap<String, String>,
    /// Color support level.
    pub color_support: ColorSupport,
    /// Unicode support information.
    pub unicode_support: UnicodeSupport,
    /// Platform information.
    pub platform: PlatformInfo,
}

impl DiagnosticReport {
    /// Generate a complete diagnostic report.
    pub fn generate() -> Self {
        Self {
            terminal: TerminalInfo::detect(),
            environment: collect_environment(),
            color_support: ColorSupport::detect(),
            unicode_support: UnicodeSupport::detect(),
            platform: PlatformInfo::detect(),
        }
    }
}

/// Detect the terminal emulator from environment variables.
fn detect_terminal_emulator() -> Option<String> {
    // Check TERM_PROGRAM first (set by many macOS terminals)
    if let Ok(program) = env::var("TERM_PROGRAM") {
        if !program.is_empty() {
            let version = env::var("TERM_PROGRAM_VERSION").unwrap_or_default();
            if !version.is_empty() {
                return Some(format!("{} {}", program, version));
            }
            return Some(program);
        }
    }

    // Check for specific terminal indicators
    if env::var("VTE_VERSION").is_ok() {
        return Some("VTE".to_string());
    }

    if env::var("GNOME_TERMINAL_SERVICE").is_ok() || env::var("GNOME_TERMINAL_SCREEN").is_ok() {
        return Some("GNOME Terminal".to_string());
    }

    if env::var("KONSOLE_VERSION").is_ok() || env::var("KONSOLE_DBUS_SERVICE").is_ok() {
        return Some("Konsole".to_string());
    }

    if env::var("ITERM_SESSION_ID").is_ok() {
        return Some("iTerm2".to_string());
    }

    if env::var("WT_SESSION").is_ok() || env::var("WT_PROFILE_ID").is_ok() {
        return Some("Windows Terminal".to_string());
    }

    if env::var("ALACRITTY_WINDOW_ID").is_ok() || env::var("ALACRITTY_SOCKET").is_ok() {
        return Some("Alacritty".to_string());
    }

    if env::var("WEZTERM_UNIX_SOCKET").is_ok() || env::var("WEZTERM_PANE").is_ok() {
        return Some("WezTerm".to_string());
    }

    if env::var("HYPER").is_ok() {
        return Some("Hyper".to_string());
    }

    if env::var("TMUX").is_ok() || env::var("TMUX_PANE").is_ok() {
        return Some("tmux".to_string());
    }

    if env::var("SCREEN").is_ok() || env::var("STY").is_ok() {
        return Some("GNU Screen".to_string());
    }

    // Check TERM for common terminal names
    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("xterm") {
            return Some("xterm-compatible".to_string());
        }
        if term_lower.contains("vt100") || term_lower.contains("vt220") {
            return Some("VT100/220".to_string());
        }
    }

    None
}

/// Check if stdout is a terminal.
fn is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Get the Rust compiler version.
fn rustc_version() -> String {
    // Try to get from environment at compile time
    option_env!("RUSTC_VERSION")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Fallback: use a compile-time version string
            format!(
                "{}.{}.{}",
                env!("CARGO_PKG_RUST_VERSION"),
                "x",
                "x"
            )
        })
}

/// Detect UTF-8 support.
fn detect_utf8_support() -> bool {
    // Check LANG and LC_ALL environment variables
    for var in ["LANG", "LC_ALL", "LC_CTYPE"] {
        if let Ok(val) = env::var(var) {
            let val_lower = val.to_lowercase();
            if val_lower.contains("utf-8") || val_lower.contains("utf8") {
                return true;
            }
        }
    }

    // On modern systems, UTF-8 is the default assumption
    true
}

/// Detect Unicode version support (approximation based on terminal capabilities).
fn detect_unicode_version() -> String {
    // This is an approximation - detecting actual Unicode version support
    // would require terminal-specific queries
    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("emoji") || term_lower.contains("modern") {
            return "14.0+ (Emoji 14.0)".to_string();
        }
    }

    // Default to a safe assumption
    "13.0+".to_string()
}

/// Detect emoji support based on terminal and environment.
fn detect_emoji_support() -> bool {
    // Most modern terminals support emoji
    // Check for explicit disabling
    if let Ok(term) = env::var("TERM") {
        if term.contains("dumb") || term.contains("linux") {
            return false;
        }
    }

    true
}

/// Detect box drawing character support.
fn detect_box_drawing_support() -> bool {
    // Most terminals support box drawing characters
    if let Ok(term) = env::var("TERM") {
        let term_lower = term.to_lowercase();
        if term_lower.contains("dumb") || term_lower.contains("vt100") {
            return false;
        }
    }

    true
}

/// Collect relevant environment variables.
fn collect_environment() -> HashMap<String, String> {
    let vars = [
        "NO_COLOR",
        "FORCE_COLOR",
        "CLICOLOR",
        "CLICOLOR_FORCE",
        "COLORTERM",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "COLUMNS",
        "LINES",
        "REDUCE_MOTION",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TMUX",
        "SCREEN",
        "VTE_VERSION",
        "ITERM_SESSION_ID",
        "WT_SESSION",
        "KONSOLE_VERSION",
        "ALACRITTY_WINDOW_ID",
        "WEZTERM_PANE",
    ];

    let mut result = HashMap::new();
    for var in &vars {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() {
                result.insert(var.to_string(), val);
            }
        }
    }

    result
}

/// Get the diagnostic report as a formatted string.
pub fn report() -> String {
    let report = DiagnosticReport::generate();
    format_report(&report)
}

/// Print the diagnostic report to the console.
pub fn print_report() {
    println!("{}", report());
}

/// Format the diagnostic report as a string.
fn format_report(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    // Header
    output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    output.push_str("║           gilt Terminal Diagnostic Report                    ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

    // Platform Information
    output.push_str("━ Platform Information ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("  OS:              {}\n", report.platform.os));
    output.push_str(&format!("  Architecture:    {}\n", report.platform.arch));
    output.push_str(&format!("  Rust Version:    {}\n", report.platform.rust_version));
    output.push_str(&format!("  gilt Version:    {}\n", report.platform.gilt_version));
    output.push('\n');

    // Terminal Information
    output.push_str("━ Terminal Information ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("  Terminal Type:   {}\n", report.terminal.term));
    output.push_str(&format!(
        "  Dimensions:      {} columns × {} rows\n",
        report.terminal.width, report.terminal.height
    ));
    output.push_str(&format!(
        "  Interactive:     {}\n",
        if report.terminal.is_terminal { "Yes" } else { "No" }
    ));
    if let Some(ref emulator) = report.terminal.emulator {
        output.push_str(&format!("  Emulator:        {}\n", emulator));
    }
    output.push('\n');

    // Color Support
    output.push_str("━ Color Support ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("  Level:           {}\n", report.color_support.name()));
    output.push_str(&format!(
        "  Standard (16):   {}\n",
        if report.color_support as u8 >= ColorSupport::Standard as u8 {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push_str(&format!(
        "  256 colors:      {}\n",
        if report.color_support as u8 >= ColorSupport::Two56 as u8 {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push_str(&format!(
        "  TrueColor:       {}\n",
        if report.color_support == ColorSupport::TrueColor {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push('\n');

    // Unicode Support
    output.push_str("━ Unicode Support ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!(
        "  UTF-8:           {}\n",
        if report.unicode_support.utf8_supported {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push_str(&format!(
        "  Version:         {}\n",
        report.unicode_support.version
    ));
    output.push_str(&format!(
        "  Emoji:           {}\n",
        if report.unicode_support.emoji_supported {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push_str(&format!(
        "  Box Drawing:     {}\n",
        if report.unicode_support.box_drawing_supported {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push_str(&format!(
        "  Wide Characters: {}\n",
        if report.unicode_support.wide_character_supported {
            "✓ Yes"
        } else {
            "✗ No"
        }
    ));
    output.push('\n');

    // Environment Variables
    output.push_str("━ Environment Variables ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Color-related variables
    output.push_str("  Color-related:\n");
    for var in ["NO_COLOR", "FORCE_COLOR", "CLICOLOR", "CLICOLOR_FORCE", "COLORTERM"] {
        if let Ok(val) = env::var(var) {
            output.push_str(&format!("    {} = \"{}\"\n", var, val));
        } else {
            output.push_str(&format!("    {} (not set)\n", var));
        }
    }

    // Size-related variables
    output.push_str("\n  Size-related:\n");
    for var in ["COLUMNS", "LINES"] {
        if let Ok(val) = env::var(var) {
            output.push_str(&format!("    {} = \"{}\"\n", var, val));
        } else {
            output.push_str(&format!("    {} (not set)\n", var));
        }
    }

    // Other variables
    output.push_str("\n  Other:\n");
    if let Ok(val) = env::var("REDUCE_MOTION") {
        output.push_str(&format!("    REDUCE_MOTION = \"{}\"\n", val));
    } else {
        output.push_str("    REDUCE_MOTION (not set)\n");
    }

    if let Ok(val) = env::var("TERM_PROGRAM") {
        output.push_str(&format!("    TERM_PROGRAM = \"{}\"\n", val));
        if let Ok(ver) = env::var("TERM_PROGRAM_VERSION") {
            output.push_str(&format!("    TERM_PROGRAM_VERSION = \"{}\"\n", ver));
        }
    }

    output.push('\n');

    // Color Test
    output.push_str("━ Color Test ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("  Standard colors: ");
    for i in 0..8 {
        output.push_str(&format!("\x1b[3{}m█\x1b[0m", i));
    }
    output.push('\n');
    output.push_str("  Bright colors:   ");
    for i in 0..8 {
        output.push_str(&format!("\x1b[9{}m█\x1b[0m", i));
    }
    output.push('\n');

    if report.color_support as u8 >= ColorSupport::Two56 as u8 {
        output.push_str("  256 colors:      ");
        for i in [0, 36, 72, 108, 144, 180, 216] {
            output.push_str(&format!("\x1b[38;5;{}m█\x1b[0m", i));
        }
        output.push('\n');
    }

    if report.color_support == ColorSupport::TrueColor {
        output.push_str("  TrueColor:       ");
        for i in 0..8 {
            let r = (i * 36) as u8;
            let g = ((7 - i) * 36) as u8;
            output.push_str(&format!("\x1b[38;2;{};{};0m█\x1b[0m", r, g));
        }
        output.push('\n');
    }

    output.push('\n');

    // Unicode Test
    output.push_str("━ Unicode Test ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("  Box drawing:     ─│┌┐└┘├┤┬┴┼\n");
    output.push_str("  Rounded:         ─│╭╮╰╯├┤┬┴┼\n");
    output.push_str("  Heavy:           ━┃┏┓┗┛┣┫┳┻╋\n");
    output.push_str("  Double:          ═║╔╗╚╝╠╣╦╩╬\n");
    output.push_str("  Emoji:           🎨 💡 🔧 📊 ✅ ❌\n");
    output.push_str("  Arrows:          ← ↑ → ↓ ↔ ↔ ↕\n");
    output.push_str("  Symbols:         ■ □ ● ○ ◆ ◇ ★ ☆\n");
    output.push('\n');

    output
}



// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_support_detection() {
        // This test just ensures detection runs without panicking
        let _support = ColorSupport::detect();
    }

    #[test]
    fn test_terminal_info_detection() {
        let info = TerminalInfo::detect();
        // Dimensions should be reasonable defaults
        assert!(info.width > 0);
        assert!(info.height > 0);
    }

    #[test]
    fn test_platform_info_detection() {
        let info = PlatformInfo::detect();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(!info.gilt_version.is_empty());
    }

    #[test]
    fn test_unicode_support_detection() {
        let support = UnicodeSupport::detect();
        // UTF-8 should be supported on most modern systems
        assert!(support.utf8_supported);
    }

    #[test]
    fn test_diagnostic_report_generation() {
        let report = DiagnosticReport::generate();
        assert!(!report.platform.os.is_empty());
        assert!(report.terminal.width > 0);
    }

    #[test]
    fn test_report_formatting() {
        let report = DiagnosticReport::generate();
        let formatted = format_report(&report);
        assert!(formatted.contains("gilt Terminal Diagnostic Report"));
        assert!(formatted.contains(&report.platform.os));
        assert!(formatted.contains(&report.platform.gilt_version));
    }

    #[test]
    fn test_color_support_name() {
        assert_eq!(ColorSupport::NoColor.name(), "No Color");
        assert_eq!(ColorSupport::Standard.name(), "Standard (16 colors)");
        assert_eq!(ColorSupport::Two56.name(), "256 colors");
        assert_eq!(ColorSupport::TrueColor.name(), "TrueColor (24-bit)");
    }

    #[test]
    fn test_color_support_color_system() {
        assert!(ColorSupport::NoColor.color_system().is_none());
        assert_eq!(
            ColorSupport::Standard.color_system(),
            Some(ColorSystem::Standard)
        );
        assert_eq!(ColorSupport::Two56.color_system(), Some(ColorSystem::EightBit));
        assert_eq!(
            ColorSupport::TrueColor.color_system(),
            Some(ColorSystem::TrueColor)
        );
    }
}
