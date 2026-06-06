//! TDD red tests for Task 1: OSC 11 terminal background detection.
//!
//! These tests are written BEFORE the implementation.
//! They should fail to compile until the implementation is in place.

use crate::color::color_triplet::ColorTriplet;
use crate::terminal_bg::{is_dark_background, parse_osc11_response, ConsoleBackground};

// ---------------------------------------------------------------------------
// parse_osc11_response tests
// ---------------------------------------------------------------------------

/// Black: all-zero channels with BEL terminator.
#[test]
fn test_parse_osc11_black_bel() {
    let result = parse_osc11_response("\x1b]11;rgb:0000/0000/0000\x07");
    assert_eq!(result, Some(ColorTriplet::new(0, 0, 0)));
}

/// White: all-max channels with BEL terminator.
#[test]
fn test_parse_osc11_white_bel() {
    let result = parse_osc11_response("\x1b]11;rgb:ffff/ffff/ffff\x07");
    assert_eq!(result, Some(ColorTriplet::new(255, 255, 255)));
}

/// String Terminator variant (`ESC \` instead of BEL).
#[test]
fn test_parse_osc11_white_st() {
    let result = parse_osc11_response("\x1b]11;rgb:ffff/ffff/ffff\x1b\\");
    assert_eq!(result, Some(ColorTriplet::new(255, 255, 255)));
}

/// 2-digit hex channels (some terminals omit padding): "ff/ff/ff" → 255,255,255.
#[test]
fn test_parse_osc11_2digit_hex() {
    let result = parse_osc11_response("\x1b]11;rgb:ff/ff/ff\x07");
    assert_eq!(result, Some(ColorTriplet::new(255, 255, 255)));
}

/// 1-digit hex channels: "f/f/f" → 255,255,255 (scale 4-bit → 8-bit).
#[test]
fn test_parse_osc11_1digit_hex() {
    let result = parse_osc11_response("\x1b]11;rgb:f/f/f\x07");
    assert_eq!(result, Some(ColorTriplet::new(255, 255, 255)));
}

/// Mid-range 4-digit channel: "8080/8080/8080" scales high byte → 0x80.
#[test]
fn test_parse_osc11_midrange() {
    let result = parse_osc11_response("\x1b]11;rgb:8080/8080/8080\x07");
    // High byte of 0x8080 is 0x80 = 128
    assert_eq!(result, Some(ColorTriplet::new(128, 128, 128)));
}

/// Malformed: no OSC prefix → None.
#[test]
fn test_parse_osc11_no_prefix_returns_none() {
    assert_eq!(parse_osc11_response("rgb:ffff/ffff/ffff"), None);
}

/// Malformed: wrong OSC number → None.
#[test]
fn test_parse_osc11_wrong_osc_number_returns_none() {
    assert_eq!(parse_osc11_response("\x1b]10;rgb:ffff/ffff/ffff\x07"), None);
}

/// Malformed: missing channel values → None.
#[test]
fn test_parse_osc11_missing_channels_returns_none() {
    assert_eq!(parse_osc11_response("\x1b]11;rgb:\x07"), None);
}

/// Malformed: non-hex characters → None.
#[test]
fn test_parse_osc11_non_hex_returns_none() {
    assert_eq!(parse_osc11_response("\x1b]11;rgb:zzzz/zzzz/zzzz\x07"), None);
}

/// Empty string → None.
#[test]
fn test_parse_osc11_empty_returns_none() {
    assert_eq!(parse_osc11_response(""), None);
}

// ---------------------------------------------------------------------------
// is_dark_background tests
// ---------------------------------------------------------------------------

/// Black background (luminance 0) → dark.
#[test]
fn test_is_dark_background_black() {
    assert!(is_dark_background(ColorTriplet::new(0, 0, 0)));
}

/// White background (luminance 1) → NOT dark.
#[test]
fn test_is_dark_background_white() {
    assert!(!is_dark_background(ColorTriplet::new(255, 255, 255)));
}

/// Mid grey exactly at the boundary: luminance of (118,118,118) is ~0.19
/// → dark (below 0.5).
#[test]
fn test_is_dark_background_mid_dark_grey() {
    // rgb(118,118,118) → luminance ~0.19 → dark
    assert!(is_dark_background(ColorTriplet::new(118, 118, 118)));
}

/// Light grey (200,200,200) → luminance ~0.59 → NOT dark.
#[test]
fn test_is_dark_background_light_grey() {
    assert!(!is_dark_background(ColorTriplet::new(200, 200, 200)));
}

// ---------------------------------------------------------------------------
// ConsoleBackground enum smoke test
// ---------------------------------------------------------------------------

/// The enum exists and the variants are distinct.
#[test]
fn test_console_background_enum_variants_exist() {
    let _dark = ConsoleBackground::Dark;
    let _light = ConsoleBackground::Light;
    let _unknown = ConsoleBackground::Unknown;
    // Ensure the variants are different (via Debug / PartialEq — both derived).
    assert_ne!(ConsoleBackground::Dark, ConsoleBackground::Light);
    assert_ne!(ConsoleBackground::Dark, ConsoleBackground::Unknown);
    assert_ne!(ConsoleBackground::Light, ConsoleBackground::Unknown);
}
