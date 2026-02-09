//! Functions for reporting file sizes in human-readable form.
//!
//! Port of Python `rich/filesize.py`, originally borrowed from
//! [pyfilesystem2](https://github.com/PyFilesystem/pyfilesystem2).
//!
//! Provides [`decimal`] to format sizes using powers of 1000 (SI prefixes)
//! and [`binary`] to format sizes using powers of 1024 (IEC binary prefixes).
//!
//! # Examples
//!
//! ```
//! use gilt::filesize::{decimal, binary};
//!
//! assert_eq!(decimal(0, 1, " "), "0 bytes");
//! assert_eq!(decimal(1, 1, " "), "1 byte");
//! assert_eq!(decimal(1000, 1, " "), "1.0 kB");
//! assert_eq!(decimal(30000, 2, ""), "30.00kB");
//!
//! assert_eq!(binary(0, 1, " "), "0 bytes");
//! assert_eq!(binary(1, 1, " "), "1 byte");
//! assert_eq!(binary(1024, 1, " "), "1.0 KiB");
//! assert_eq!(binary(30000, 2, ""), "29.30KiB");
//! ```

/// SI suffixes used by [`decimal`].
const DECIMAL_SUFFIXES: &[&str] = &["kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

/// IEC binary suffixes used by [`binary`].
const BINARY_SUFFIXES: &[&str] = &["KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];

/// Format an integer with comma-separated thousands.
///
/// ```text
/// 0        -> "0"
/// 999      -> "999"
/// 1000     -> "1,000"
/// 1000000  -> "1,000,000"
/// ```
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    if len <= 3 {
        return s;
    }
    let mut result = String::with_capacity(len + (len - 1) / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Format a floating-point value with comma-separated thousands in the integer
/// part, and exactly `precision` decimal places.
fn format_float_with_commas(value: f64, precision: usize) -> String {
    // Format the number with the desired precision first to get correct rounding.
    let formatted = format!("{value:.prec$}", prec = precision);

    // Split into integer and fractional parts.
    let (int_part, frac_part) = match formatted.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (formatted.as_str(), None),
    };

    let len = int_part.len();
    let mut result = String::with_capacity(len + (len.saturating_sub(1)) / 3 + 1 + precision);

    for (i, ch) in int_part.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    if let Some(frac) = frac_part {
        result.push('.');
        result.push_str(frac);
    }

    result
}

/// Compute `base.pow(exp)` as `f64`, avoiding `u64` overflow for large exponents.
fn base_pow_f64(base: u64, exp: u32) -> f64 {
    (base as f64).powi(exp as i32)
}

/// Internal formatting helper shared by public functions.
///
/// Handles the three cases:
/// - size == 1 -> "1 byte"
/// - size < base -> "{size} bytes" (comma-formatted)
/// - otherwise -> find appropriate unit and format with precision
///
/// Mirrors Python's `_to_str` which uses `enumerate(suffixes, 2)`, meaning
/// the first suffix corresponds to `base^2`, the second to `base^3`, etc.
fn to_str(size: u64, suffixes: &[&str], base: u64, precision: usize, separator: &str) -> String {
    if size == 1 {
        return "1 byte".to_string();
    }
    if size < base {
        return format!("{} bytes", format_with_commas(size));
    }

    // Enumerate starting at exponent 2, matching Python's enumerate(suffixes, 2).
    // Use f64 for unit computation to avoid u64 overflow at large exponents.
    let size_f = size as f64;
    let mut unit_f = 1.0_f64;
    let mut suffix = suffixes.last().copied().unwrap_or("");
    for (idx, s) in suffixes.iter().enumerate() {
        let exp = (idx + 2) as u32;
        unit_f = base_pow_f64(base, exp);
        if size_f < unit_f {
            suffix = s;
            break;
        }
    }

    let value = (base as f64) * size_f / unit_f;
    format!(
        "{}{}{}",
        format_float_with_commas(value, precision),
        separator,
        suffix,
    )
}

/// Pick the appropriate unit and suffix for a given size.
///
/// Iterates through `suffixes` with their index; `unit = base.pow(index)`.
/// Returns the first `(unit, suffix)` where `size < unit * base`.
///
/// Note: the returned suffix is the one at the iteration index where the
/// condition was satisfied, which may not semantically correspond to `unit`.
/// This matches the Python `rich` behaviour.
///
/// # Examples
///
/// ```
/// use gilt::filesize::pick_unit_and_suffix;
///
/// let suffixes = &["kB", "MB", "GB"];
/// let (unit, suffix) = pick_unit_and_suffix(500, suffixes, 1000);
/// assert_eq!(unit, 1);
/// assert_eq!(suffix, "kB");
/// ```
pub fn pick_unit_and_suffix(size: u64, suffixes: &[&str], base: u64) -> (u64, String) {
    let mut unit = 1u64;
    let mut chosen_suffix = suffixes.last().copied().unwrap_or("");

    for (i, suffix) in suffixes.iter().enumerate() {
        unit = base.pow(i as u32);
        if size < unit.saturating_mul(base) {
            chosen_suffix = suffix;
            break;
        }
    }

    (unit, chosen_suffix.to_string())
}

/// Convert a file size to a human-readable string using powers of 1000
/// (SI prefixes).
///
/// In this convention, `1000 B = 1 kB`.
///
/// This is typically the format used to advertise the storage capacity of
/// USB flash drives and similar devices, or used by **macOS** since v10.6
/// to report file sizes.
///
/// # Arguments
///
/// * `size` - The file size in bytes.
/// * `precision` - Number of decimal places (typically 1).
/// * `separator` - String placed between the value and the unit (typically `" "`).
///
/// # Examples
///
/// ```
/// use gilt::filesize::decimal;
///
/// assert_eq!(decimal(30000, 1, " "), "30.0 kB");
/// assert_eq!(decimal(30000, 2, ""), "30.00kB");
/// ```
pub fn decimal(size: u64, precision: usize, separator: &str) -> String {
    to_str(size, DECIMAL_SUFFIXES, 1000, precision, separator)
}

/// Convert a file size to a human-readable string using powers of 1024
/// (IEC binary prefixes).
///
/// In this convention, `1024 B = 1 KiB`.
///
/// This is typically the format used by **Linux** to report file sizes and by
/// tools like `du -h` or `ls -lh`.
///
/// # Arguments
///
/// * `size` - The file size in bytes.
/// * `precision` - Number of decimal places (typically 1).
/// * `separator` - String placed between the value and the unit (typically `" "`).
///
/// # Examples
///
/// ```
/// use gilt::filesize::binary;
///
/// assert_eq!(binary(1024, 1, " "), "1.0 KiB");
/// assert_eq!(binary(30000, 2, ""), "29.30KiB");
/// ```
pub fn binary(size: u64, precision: usize, separator: &str) -> String {
    to_str(size, BINARY_SUFFIXES, 1024, precision, separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_with_commas ──────────────────────────────────────────

    #[test]
    fn commas_zero() {
        assert_eq!(format_with_commas(0), "0");
    }

    #[test]
    fn commas_small() {
        assert_eq!(format_with_commas(999), "999");
    }

    #[test]
    fn commas_thousands() {
        assert_eq!(format_with_commas(1_000), "1,000");
    }

    #[test]
    fn commas_millions() {
        assert_eq!(format_with_commas(1_000_000), "1,000,000");
    }

    #[test]
    fn commas_large() {
        assert_eq!(format_with_commas(1_234_567_890), "1,234,567,890");
    }

    // ── decimal: special cases ──────────────────────────────────────

    #[test]
    fn decimal_zero_bytes() {
        assert_eq!(decimal(0, 1, " "), "0 bytes");
    }

    #[test]
    fn decimal_one_byte() {
        assert_eq!(decimal(1, 1, " "), "1 byte");
    }

    #[test]
    fn decimal_small_bytes() {
        assert_eq!(decimal(999, 1, " "), "999 bytes");
    }

    #[test]
    fn decimal_bytes_with_commas() {
        // With base 1000, anything under 1000 is plain (no commas).
        assert_eq!(decimal(500, 1, " "), "500 bytes");
    }

    // ── decimal: kB range ───────────────────────────────────────────

    #[test]
    fn decimal_exactly_1000() {
        assert_eq!(decimal(1000, 1, " "), "1.0 kB");
    }

    #[test]
    fn decimal_1500() {
        assert_eq!(decimal(1500, 1, " "), "1.5 kB");
    }

    #[test]
    fn decimal_30000() {
        assert_eq!(decimal(30000, 1, " "), "30.0 kB");
    }

    #[test]
    fn decimal_999999() {
        assert_eq!(decimal(999_999, 1, " "), "1,000.0 kB");
    }

    // ── decimal: MB range ───────────────────────────────────────────

    #[test]
    fn decimal_one_million() {
        assert_eq!(decimal(1_000_000, 1, " "), "1.0 MB");
    }

    #[test]
    fn decimal_1_500_000() {
        assert_eq!(decimal(1_500_000, 1, " "), "1.5 MB");
    }

    // ── decimal: GB range ───────────────────────────────────────────

    #[test]
    fn decimal_one_billion() {
        assert_eq!(decimal(1_000_000_000, 1, " "), "1.0 GB");
    }

    // ── decimal: TB range ───────────────────────────────────────────

    #[test]
    fn decimal_one_trillion() {
        assert_eq!(decimal(1_000_000_000_000, 1, " "), "1.0 TB");
    }

    // ── decimal: custom precision ───────────────────────────────────

    #[test]
    fn decimal_precision_2() {
        assert_eq!(decimal(30000, 2, " "), "30.00 kB");
    }

    #[test]
    fn decimal_precision_0() {
        assert_eq!(decimal(30000, 0, " "), "30 kB");
    }

    // ── decimal: custom separator ───────────────────────────────────

    #[test]
    fn decimal_empty_separator() {
        assert_eq!(decimal(30000, 2, ""), "30.00kB");
    }

    #[test]
    fn decimal_dash_separator() {
        assert_eq!(decimal(30000, 1, "-"), "30.0-kB");
    }

    // ── decimal: very large ─────────────────────────────────────────

    #[test]
    fn decimal_petabyte() {
        assert_eq!(decimal(1_000_000_000_000_000, 1, " "), "1.0 PB");
    }

    #[test]
    fn decimal_exabyte() {
        assert_eq!(decimal(1_000_000_000_000_000_000, 1, " "), "1.0 EB");
    }

    // ── pick_unit_and_suffix ────────────────────────────────────────

    #[test]
    fn pick_unit_small_size() {
        // size=500, base=1000: i=0, unit=1, 500 < 1*1000 -> true
        let suffixes = &["kB", "MB", "GB", "TB"];
        let (unit, suffix) = pick_unit_and_suffix(500, suffixes, 1000);
        assert_eq!(unit, 1);
        assert_eq!(suffix, "kB");
    }

    #[test]
    fn pick_unit_medium_size() {
        // size=1,500,000: i=0 no, i=1 no, i=2 unit=1M, 1.5M < 1B -> true
        let suffixes = &["kB", "MB", "GB", "TB"];
        let (unit, suffix) = pick_unit_and_suffix(1_500_000, suffixes, 1000);
        assert_eq!(unit, 1_000_000);
        assert_eq!(suffix, "GB");
    }

    #[test]
    fn pick_unit_large_size() {
        // size=5,000,000,000: i=0 no, i=1 no, i=2 no, i=3 unit=1B, 5B < 1T -> true
        let suffixes = &["kB", "MB", "GB", "TB"];
        let (unit, suffix) = pick_unit_and_suffix(5_000_000_000, suffixes, 1000);
        assert_eq!(unit, 1_000_000_000);
        assert_eq!(suffix, "TB");
    }

    #[test]
    fn pick_unit_falls_through() {
        // size=999,000,000,000 with only ["kB", "MB"]:
        // i=0: unit=1, 999B < 1000 -> false
        // i=1: unit=1000, 999B < 1,000,000 -> false
        // Falls through, unit stays 1000, suffix stays "MB"
        let suffixes = &["kB", "MB"];
        let (unit, suffix) = pick_unit_and_suffix(999_000_000_000, suffixes, 1000);
        assert_eq!(unit, 1000);
        assert_eq!(suffix, "MB");
    }

    // ── binary: special cases ──────────────────────────────────────

    #[test]
    fn binary_zero_bytes() {
        assert_eq!(binary(0, 1, " "), "0 bytes");
    }

    #[test]
    fn binary_one_byte() {
        assert_eq!(binary(1, 1, " "), "1 byte");
    }

    #[test]
    fn binary_small_bytes() {
        assert_eq!(binary(1023, 1, " "), "1,023 bytes");
    }

    // ── binary: KiB range ───────────────────────────────────────────

    #[test]
    fn binary_exactly_1024() {
        assert_eq!(binary(1024, 1, " "), "1.0 KiB");
    }

    #[test]
    fn binary_1536() {
        assert_eq!(binary(1536, 1, " "), "1.5 KiB");
    }

    #[test]
    fn binary_30000() {
        assert_eq!(binary(30000, 2, " "), "29.30 KiB");
    }

    // ── binary: MiB range ───────────────────────────────────────────

    #[test]
    fn binary_one_mebibyte() {
        assert_eq!(binary(1_048_576, 1, " "), "1.0 MiB");
    }

    #[test]
    fn binary_1_5_mebibytes() {
        assert_eq!(binary(1_572_864, 1, " "), "1.5 MiB");
    }

    // ── binary: GiB range ───────────────────────────────────────────

    #[test]
    fn binary_one_gibibyte() {
        assert_eq!(binary(1_073_741_824, 1, " "), "1.0 GiB");
    }

    // ── binary: TiB range ───────────────────────────────────────────

    #[test]
    fn binary_one_tebibyte() {
        assert_eq!(binary(1_099_511_627_776, 1, " "), "1.0 TiB");
    }

    // ── binary: custom precision ───────────────────────────────────

    #[test]
    fn binary_precision_2() {
        assert_eq!(binary(30000, 2, " "), "29.30 KiB");
    }

    #[test]
    fn binary_precision_0() {
        assert_eq!(binary(30000, 0, " "), "29 KiB");
    }

    // ── binary: custom separator ───────────────────────────────────

    #[test]
    fn binary_empty_separator() {
        assert_eq!(binary(30000, 2, ""), "29.30KiB");
    }

    #[test]
    fn binary_dash_separator() {
        assert_eq!(binary(30000, 1, "-"), "29.3-KiB");
    }

    // ── binary: very large ─────────────────────────────────────────

    #[test]
    fn binary_pebibyte() {
        assert_eq!(binary(1_125_899_906_842_624, 1, " "), "1.0 PiB");
    }

    #[test]
    fn binary_exbibyte() {
        assert_eq!(binary(1_152_921_504_606_846_976, 1, " "), "1.0 EiB");
    }
}
