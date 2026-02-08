//! Environment variable detection for color overrides.
//!
//! Supports the following environment variables (checked in priority order):
//!
//! 1. **`NO_COLOR`** – Any value disables color (<https://no-color.org/>)
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
/// 1. `NO_COLOR` (any value) → [`ColorEnvOverride::NoColor`]
/// 2. `FORCE_COLOR`:
///    - `"0"` → [`ColorEnvOverride::NoColor`]
///    - `"1"` | `"2"` → [`ColorEnvOverride::ForceColor`]
///    - `"3"` → [`ColorEnvOverride::ForceColorTruecolor`]
///    - any other value → [`ColorEnvOverride::ForceColor`]
/// 3. `CLICOLOR_FORCE` (any non-`"0"` value) → [`ColorEnvOverride::ForceColor`]
/// 4. `CLICOLOR` = `"0"` → [`ColorEnvOverride::NoColor`]
/// 5. Otherwise → [`ColorEnvOverride::None`]
pub fn detect_color_env() -> ColorEnvOverride {
    // 1. NO_COLOR – presence alone is enough
    if env::var_os("NO_COLOR").is_some() {
        return ColorEnvOverride::NoColor;
    }

    // 2. FORCE_COLOR
    if let Ok(val) = env::var("FORCE_COLOR") {
        return match val.as_str() {
            "0" => ColorEnvOverride::NoColor,
            "1" | "2" => ColorEnvOverride::ForceColor,
            "3" => ColorEnvOverride::ForceColorTruecolor,
            _ => ColorEnvOverride::ForceColor,
        };
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
        let saved: Vec<(&str, Option<String>)> = all_keys
            .iter()
            .map(|k| (*k, env::var(k).ok()))
            .collect();

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
        let r = with_env(&[("NO_COLOR", Some(""))], detect_color_env);
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
        let r = with_env(
            &[("NO_COLOR", Some("")), ("FORCE_COLOR", Some("3"))],
            detect_color_env,
        );
        assert_eq!(r, ColorEnvOverride::NoColor);
    }

    #[test]
    fn test_force_color_wins_over_clicolor_force() {
        let r = with_env(
            &[
                ("FORCE_COLOR", Some("0")),
                ("CLICOLOR_FORCE", Some("1")),
            ],
            detect_color_env,
        );
        // FORCE_COLOR=0 → NoColor, even though CLICOLOR_FORCE=1
        assert_eq!(r, ColorEnvOverride::NoColor);
    }
}
