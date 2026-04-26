//! Environment variable detection for color overrides.
//!
//! Supports the following environment variables (checked in priority order):
//!
//! 1. **`NO_COLOR`** – A *non-empty* value disables color (<https://no-color.org/>)
//! 2. **`FORCE_COLOR`** – Node.js convention: `0` = off, `1`/`2` = standard/256, `3` = truecolor
//! 3. **`CLICOLOR_FORCE`** – Any non-`"0"` value forces color on
//! 4. **`CLICOLOR`** – `"0"` disables color
//!
//! These are only consulted when the user hasn't explicitly set `no_color` or
//! `color_system` on the [`ConsoleBuilder`](crate::console::ConsoleBuilder).

use std::env;

/// The recommendation produced by [`detect_color_env`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEnvOverride {
    /// Color should be disabled (e.g. `NO_COLOR` is set).
    NoColor,
    /// Color should be forced on (at least standard/256).
    ForceColor,
    /// Color should be forced to truecolor (24-bit).
    ForceColorTruecolor,
    /// No environment override detected – use normal detection.
    None,
}

/// Inspect environment variables and return a color override recommendation.
///
/// Priority (highest first):
/// 1. `NO_COLOR` (non-empty value) → [`ColorEnvOverride::NoColor`]
/// 2. `FORCE_COLOR`:
///    - `"0"` → [`ColorEnvOverride::NoColor`]
///    - `"1"` | `"2"` → [`ColorEnvOverride::ForceColor`]
///    - `"3"` → [`ColorEnvOverride::ForceColorTruecolor`]
///    - any other non-empty value → [`ColorEnvOverride::ForceColor`]
///    - `""` (empty) → no force (falls through)
/// 3. `CLICOLOR_FORCE` (any non-`"0"` value) → [`ColorEnvOverride::ForceColor`]
/// 4. `CLICOLOR` = `"0"` → [`ColorEnvOverride::NoColor`]
/// 5. Otherwise → [`ColorEnvOverride::None`]
pub fn detect_color_env() -> ColorEnvOverride {
    // 1. NO_COLOR – only a *non-empty* value disables color (rich v14 semantics).
    // An empty `NO_COLOR=""` is treated as unset, per https://no-color.org/ and
    // upstream rich commit a919527f.
    if env::var("NO_COLOR").map_or(false, |v| !v.is_empty()) {
        return ColorEnvOverride::NoColor;
    }

    // 2. FORCE_COLOR – empty string does NOT force color (rich v14 semantics,
    // upstream commit 9175392a).  Only a non-empty value is acted upon.
    if let Ok(val) = env::var("FORCE_COLOR") {
        match val.as_str() {
            "" => {} // empty → no override; fall through to next variable
            "0" => return ColorEnvOverride::NoColor,
            "1" | "2" => return ColorEnvOverride::ForceColor,
            "3" => return ColorEnvOverride::ForceColorTruecolor,
            _ => return ColorEnvOverride::ForceColor,
        }
    }

    // 3. CLICOLOR_FORCE – any non-"0" value forces color
    if let Ok(val) = env::var("CLICOLOR_FORCE") {
        if val != "0" {
            return ColorEnvOverride::ForceColor;
        }
    }

    // 4. CLICOLOR=0 disables color
    if let Ok(val) = env::var("CLICOLOR") {
        if val == "0" {
            return ColorEnvOverride::NoColor;
        }
    }

    ColorEnvOverride::None
}

/// Detect if the user prefers reduced motion.
///
/// Returns `true` if the `REDUCE_MOTION` environment variable is set to
/// `"1"` or `"true"` (case-insensitive).
///
/// This allows applications to skip animations (spinners, progress bars,
/// live updates) when the user has expressed a preference for reduced motion.
pub fn detect_reduce_motion() -> bool {
    match env::var("REDUCE_MOTION") {
        Ok(val) => val == "1" || val.eq_ignore_ascii_case("true"),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Env-var tests must be serialised because env is process-global.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper: remove all color env vars, run `f`, then restore.
    fn with_env<F: FnOnce() -> ColorEnvOverride>(
        vars: &[(&str, Option<&str>)],
        f: F,
    ) -> ColorEnvOverride {
        let _guard = ENV_LOCK.lock().unwrap();

        // Save originals & clear all colour vars first.
        let all_keys = ["NO_COLOR", "FORCE_COLOR", "CLICOLOR_FORCE", "CLICOLOR"];
        let saved: Vec<(&str, Option<String>)> =
            all_keys.iter().map(|k| (*k, env::var(k).ok())).collect();

        for key in &all_keys {
            env::remove_var(key);
        }

        // Set requested vars.
        for &(key, val) in vars {
            match val {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }

        let result = f();

        // Restore.
        for (key, val) in saved {
            match val {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }

        result
    }

    #[test]
    fn test_no_color_set_disables_color() {
        // Non-empty NO_COLOR disables color.
        let r = with_env(&[("NO_COLOR", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_no_color_any_value() {
        let r = with_env(&[("NO_COLOR", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_force_color_3_truecolor() {
        let r = with_env(&[("FORCE_COLOR", Some("3"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::ForceColorTruecolor);
    }

    #[test]
    fn test_force_color_0_disables() {
        let r = with_env(&[("FORCE_COLOR", Some("0"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_force_color_1_forces() {
        let r = with_env(&[("FORCE_COLOR", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::ForceColor);
    }

    #[test]
    fn test_force_color_unknown_value_forces() {
        let r = with_env(&[("FORCE_COLOR", Some("yes"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::ForceColor);
    }

    #[test]
    fn test_clicolor_force_1() {
        let r = with_env(&[("CLICOLOR_FORCE", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::ForceColor);
    }

    #[test]
    fn test_clicolor_force_0_does_not_force() {
        // CLICOLOR_FORCE=0 is a no-op; falls through to None.
        let r = with_env(&[("CLICOLOR_FORCE", Some("0"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::None);
    }

    #[test]
    fn test_clicolor_0_disables() {
        let r = with_env(&[("CLICOLOR", Some("0"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_clicolor_1_no_override() {
        // CLICOLOR=1 doesn't force anything; it's the default.
        let r = with_env(&[("CLICOLOR", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::None);
    }

    #[test]
    fn test_no_vars_set_returns_none() {
        let r = with_env(&[], detect_color_env);
        assert_eq!(r, ColorEnvOverride::None);
    }

    #[test]
    fn test_no_color_wins_over_force_color() {
        // Non-empty NO_COLOR takes priority over FORCE_COLOR.
        let r = with_env(
            &[("NO_COLOR", Some("1")), ("FORCE_COLOR", Some("3"))],
            detect_color_env,
        );
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_force_color_wins_over_clicolor_force() {
        let r = with_env(
            &[("FORCE_COLOR", Some("0")), ("CLICOLOR_FORCE", Some("1"))],
            detect_color_env,
        );
        // FORCE_COLOR=0 → NoColor, even though CLICOLOR_FORCE=1
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    // --- detect_reduce_motion tests ---

    /// Helper for REDUCE_MOTION tests: clears REDUCE_MOTION, sets `val`, runs `f`, restores.
    fn with_reduce_motion<F: FnOnce() -> bool>(val: Option<&str>, f: F) -> bool {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = env::var("REDUCE_MOTION").ok();
        env::remove_var("REDUCE_MOTION");
        if let Some(v) = val {
            env::set_var("REDUCE_MOTION", v);
        }
        let result = f();
        match saved {
            Some(v) => env::set_var("REDUCE_MOTION", v),
            None => env::remove_var("REDUCE_MOTION"),
        }
        result
    }

    #[test]
    fn test_reduce_motion_unset() {
        let r = with_reduce_motion(None, super::detect_reduce_motion);
        assert!(!r, "should be false when REDUCE_MOTION is not set");
    }

    #[test]
    fn test_reduce_motion_1() {
        let r = with_reduce_motion(Some("1"), super::detect_reduce_motion);
        assert!(r, "should be true when REDUCE_MOTION=1");
    }

    #[test]
    fn test_reduce_motion_true_lowercase() {
        let r = with_reduce_motion(Some("true"), super::detect_reduce_motion);
        assert!(r, "should be true when REDUCE_MOTION=true");
    }

    #[test]
    fn test_reduce_motion_true_uppercase() {
        let r = with_reduce_motion(Some("TRUE"), super::detect_reduce_motion);
        assert!(r, "should be true when REDUCE_MOTION=TRUE");
    }

    #[test]
    fn test_reduce_motion_true_mixed_case() {
        let r = with_reduce_motion(Some("True"), super::detect_reduce_motion);
        assert!(r, "should be true when REDUCE_MOTION=True");
    }

    #[test]
    fn test_reduce_motion_0() {
        let r = with_reduce_motion(Some("0"), super::detect_reduce_motion);
        assert!(!r, "should be false when REDUCE_MOTION=0");
    }

    #[test]
    fn test_reduce_motion_empty() {
        let r = with_reduce_motion(Some(""), super::detect_reduce_motion);
        assert!(!r, "should be false when REDUCE_MOTION is empty");
    }

    #[test]
    fn test_reduce_motion_arbitrary_value() {
        let r = with_reduce_motion(Some("yes"), super::detect_reduce_motion);
        assert!(!r, "should be false for arbitrary values like 'yes'");
    }

    // --- rich v14 empty-string semantics ---

    #[test]
    fn no_color_unset_means_color_enabled() {
        // When NO_COLOR is not in the environment at all, color is not disabled.
        let r = with_env(&[], detect_color_env);
        assert_ne!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn no_color_empty_string_does_not_disable() {
        // Empty NO_COLOR="" must NOT disable color (rich v14 / no-color.org semantics).
        let r = with_env(&[("NO_COLOR", Some(""))], detect_color_env);
        assert_ne!(
            r,
            ColorEnvOverride::NoColor,
            "NO_COLOR='' should not disable color"
        );
    }

    #[test]
    fn no_color_nonempty_disables() {
        // Any non-empty value (even just a space) disables color.
        let r = with_env(&[("NO_COLOR", Some("yes"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn force_color_unset_no_force() {
        // Absent FORCE_COLOR should not force color on.
        let r = with_env(&[], detect_color_env);
        assert_eq!(r, ColorEnvOverride::None);
    }

    #[test]
    fn force_color_empty_string_does_not_force() {
        // Empty FORCE_COLOR="" must NOT force color (rich v14 semantics).
        let r = with_env(&[("FORCE_COLOR", Some(""))], detect_color_env);
        assert_eq!(
            r,
            ColorEnvOverride::None,
            "FORCE_COLOR='' should not force color"
        );
    }

    #[test]
    fn force_color_nonempty_forces() {
        // Non-empty FORCE_COLOR="1" forces color on.
        let r = with_env(&[("FORCE_COLOR", Some("1"))], detect_color_env);
        assert_eq!(r, ColorEnvOverride::ForceColor);
    }
}
