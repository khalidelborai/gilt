//! OSC 11 terminal background detection — auto dark/light theme selection.
//!
//! # Core API (always available, dep-free, WASM-safe)
//!
//! - [`parse_osc11_response`] — parse a terminal's `\x1b]11;rgb:RRRR/GGGG/BBBB\x07` reply.
//! - [`is_dark_background`] — decide whether a colour is perceived as dark (luminance < 0.5).
//! - [`ConsoleBackground`] — `Dark / Light / Unknown` enum.
//!
//! # Interactive probe (feature `terminal-query`, native only)
//!
//! When the `terminal-query` feature is enabled **and** the build target is not
//! `wasm32`, the function [`query_terminal_background`] is available.  It writes
//! an OSC 11 query to the controlling TTY, waits up to 200 ms for the response,
//! and returns the parsed `ConsoleBackground`.  Any I/O error or timeout yields
//! `ConsoleBackground::Unknown`.
//!
//! # Example
//!
//! ```
//! use gilt::terminal_bg::{parse_osc11_response, is_dark_background, ConsoleBackground};
//! use gilt::color::color_triplet::ColorTriplet;
//!
//! // Parse a typical Xterm OSC 11 reply for a dark background.
//! let triplet = parse_osc11_response("\x1b]11;rgb:1a1a/1a1a/1a1a\x07").unwrap();
//! assert!(is_dark_background(triplet));
//!
//! // Parse a light background reply.
//! let white = parse_osc11_response("\x1b]11;rgb:ffff/ffff/ffff\x07").unwrap();
//! assert!(!is_dark_background(white));
//! ```

use crate::color::color_triplet::ColorTriplet;

// ---------------------------------------------------------------------------
// ConsoleBackground
// ---------------------------------------------------------------------------

/// Whether the terminal background is perceived as dark, light, or unknown.
///
/// Returned by [`Console::detect_background`](crate::console::Console::detect_background)
/// and (on native targets with the `terminal-query` feature) by
/// [`query_terminal_background`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleBackground {
    /// Background luminance < 0.5 (dark terminal theme).
    Dark,
    /// Background luminance >= 0.5 (light terminal theme).
    Light,
    /// Could not determine the background (no TTY, timeout, unsupported terminal).
    Unknown,
}

// ---------------------------------------------------------------------------
// Core parsing (dep-free, WASM-safe)
// ---------------------------------------------------------------------------

/// Parse a terminal's OSC 11 background-colour response.
///
/// Accepts replies terminated by either BEL (`\x07`) or String Terminator
/// (`\x1b\\`).  Channels may be 1, 2, or 4 hex digits; they are scaled to
/// 8-bit by taking the high byte:
///
/// | Digits | Example   | Scaled 8-bit |
/// |--------|-----------|--------------|
/// | 1      | `f`       | `0xff` → 255 |
/// | 2      | `ff`      | `0xff` → 255 |
/// | 4      | `ffff`    | high byte `0xff` → 255 |
/// | 4      | `8080`    | high byte `0x80` → 128 |
///
/// Returns `None` on any parse error (missing prefix, wrong OSC number,
/// non-hex digits, wrong channel count, etc.).
///
/// # Examples
///
/// ```
/// use gilt::terminal_bg::parse_osc11_response;
///
/// // Black background
/// assert_eq!(
///     parse_osc11_response("\x1b]11;rgb:0000/0000/0000\x07"),
///     Some(gilt::color::color_triplet::ColorTriplet::new(0, 0, 0))
/// );
///
/// // White background
/// assert_eq!(
///     parse_osc11_response("\x1b]11;rgb:ffff/ffff/ffff\x07"),
///     Some(gilt::color::color_triplet::ColorTriplet::new(255, 255, 255))
/// );
///
/// // Malformed → None
/// assert_eq!(parse_osc11_response("not-an-osc-reply"), None);
/// ```
pub fn parse_osc11_response(s: &str) -> Option<ColorTriplet> {
    // Expected shape:  \x1b ] 11 ; rgb: RR / GG / BB \x07
    //   or with ST:    \x1b ] 11 ; rgb: RR / GG / BB \x1b \\

    // Must start with ESC ]
    let rest = s.strip_prefix("\x1b]")?;

    // Strip the trailing terminator first, then parse the body.
    // Accepted terminators: BEL (\x07) or ST (\x1b\).
    let body = if let Some(b) = rest.strip_suffix('\x07') {
        b
    } else {
        rest.strip_suffix("\x1b\\")?
    };

    // Body must be "11;rgb:..."
    let rgb_part = body.strip_prefix("11;rgb:")?;

    // Split the three channel tokens at '/'
    let mut parts = rgb_part.splitn(4, '/');
    let r_str = parts.next()?;
    let g_str = parts.next()?;
    let b_str = parts.next()?;
    // Reject any extra content after the third channel
    if parts.next().is_some() {
        return None;
    }

    let r = parse_channel(r_str)?;
    let g = parse_channel(g_str)?;
    let b = parse_channel(b_str)?;

    Some(ColorTriplet::new(r, g, b))
}

/// Scale a 1/2/4-digit hex channel string to an 8-bit value.
///
/// - 4 digits: take the high byte (first 2 digits).
/// - 2 digits: direct 8-bit value.
/// - 1 digit:  replicate to both nibbles (e.g. `f` → `ff`).
///
/// Returns `None` for empty, non-hex, or unsupported lengths.
fn parse_channel(hex: &str) -> Option<u8> {
    match hex.len() {
        4 => {
            // 16-bit channel: take the high byte (first 2 hex digits)
            u8::from_str_radix(&hex[..2], 16).ok()
        }
        2 => u8::from_str_radix(hex, 16).ok(),
        1 => {
            // 1 nibble: replicate (0xN → 0xNN)
            let nibble = u8::from_str_radix(hex, 16).ok()?;
            Some(nibble << 4 | nibble)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Luminance check (reuses the WCAG formula in color/accessibility.rs)
// ---------------------------------------------------------------------------

/// Return `true` if the colour is perceived as dark (WCAG relative luminance < 0.5).
///
/// Uses the same linearisation formula as [`crate::accessibility::contrast_ratio`].
///
/// # Examples
///
/// ```
/// use gilt::terminal_bg::is_dark_background;
/// use gilt::color::color_triplet::ColorTriplet;
///
/// assert!(is_dark_background(ColorTriplet::new(0, 0, 0)));    // black → dark
/// assert!(!is_dark_background(ColorTriplet::new(255, 255, 255))); // white → light
/// ```
pub fn is_dark_background(c: ColorTriplet) -> bool {
    relative_luminance(c) < 0.5
}

/// WCAG 2.1 relative luminance for a single colour.
///
/// Duplicated here (rather than re-exported from `accessibility`) so
/// `terminal_bg` remains dep-free and can be compiled without pulling in the
/// rest of `color::accessibility`.
fn relative_luminance(color: ColorTriplet) -> f64 {
    let (r, g, b) = color.normalized();
    let linearize = |c: f64| -> f64 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

// ---------------------------------------------------------------------------
// Interactive TTY probe  (feature = "terminal-query", native only)
// ---------------------------------------------------------------------------

/// Query the controlling terminal for its background colour using OSC 11.
///
/// Writes `\x1b]11;?\x07` to the TTY, then reads back the reply with a
/// 200 ms timeout in raw mode.  On success the response is parsed via
/// [`parse_osc11_response`] and converted to a [`ConsoleBackground`] via
/// [`is_dark_background`].
///
/// Returns [`ConsoleBackground::Unknown`] when:
/// - No controlling TTY is available.
/// - The terminal does not support OSC 11 queries.
/// - The timeout expires before a complete reply arrives.
/// - Any I/O error occurs.
///
/// Requires the `terminal-query` feature and is excluded from WASM builds.
#[cfg(all(feature = "terminal-query", not(target_arch = "wasm32")))]
pub fn query_terminal_background() -> ConsoleBackground {
    use crossterm::{
        event::{poll, read, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };
    use std::io::Write as _;
    use std::time::Duration;

    // Open /dev/tty (or equivalent) directly so we don't disturb stdout/stdin.
    let tty_path = if cfg!(unix) { "/dev/tty" } else { "CON" };
    let Ok(mut tty) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(tty_path)
    else {
        return ConsoleBackground::Unknown;
    };

    // Enable raw mode so we can read the escape response byte-by-byte.
    if enable_raw_mode().is_err() {
        return ConsoleBackground::Unknown;
    }

    // Send the OSC 11 query.
    if tty.write_all(b"\x1b]11;?\x07").is_err() || tty.flush().is_err() {
        let _ = disable_raw_mode();
        return ConsoleBackground::Unknown;
    }

    // Accumulate the response, stopping at BEL (\x07) or ST (\x1b\\).
    let mut response = String::with_capacity(32);
    let timeout = Duration::from_millis(200);
    let result = (|| {
        loop {
            if !poll(timeout).unwrap_or(false) {
                return ConsoleBackground::Unknown;
            }
            match read() {
                Ok(Event::Key(key)) => {
                    // crossterm decodes key events; we need raw bytes.
                    // Build the OSC string from the key character.
                    let ch = match key.code {
                        KeyCode::Char(c) => c,
                        _ => return ConsoleBackground::Unknown,
                    };
                    response.push(ch);
                    if ch == '\x07' || response.ends_with("\x1b\\") {
                        break;
                    }
                }
                _ => return ConsoleBackground::Unknown,
            }
        }
        // Prepend the OSC introducer that crossterm ate.
        let full = format!("\x1b]{response}");
        match parse_osc11_response(&full) {
            Some(triplet) => {
                if is_dark_background(triplet) {
                    ConsoleBackground::Dark
                } else {
                    ConsoleBackground::Light
                }
            }
            None => ConsoleBackground::Unknown,
        }
    })();

    let _ = disable_raw_mode();
    result
}
