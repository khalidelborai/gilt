//! Demonstrates environment variable color detection with `detect_color_env()`.
//!
//! gilt respects a 5-tier priority chain of environment variables that
//! control whether terminal output is colored:
//!
//! 1. `NO_COLOR`       — Any value disables color (https://no-color.org/)
//! 2. `FORCE_COLOR`    — `0` = off, `1`/`2` = standard/256, `3` = truecolor
//! 3. `CLICOLOR_FORCE` — Any non-`"0"` value forces color on
//! 4. `CLICOLOR`       — `"0"` disables color
//! 5. (none set)       — Normal terminal auto-detection
//!
//! Try running with different env vars to see the effect:
//!
//!   cargo run --example color_env
//!   NO_COLOR=1 cargo run --example color_env
//!   FORCE_COLOR=3 cargo run --example color_env
//!   FORCE_COLOR=0 cargo run --example color_env
//!   CLICOLOR=0 cargo run --example color_env
//!   CLICOLOR_FORCE=1 cargo run --example color_env

use gilt::color_env::{detect_color_env, ColorEnvOverride};
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::styled_str::Stylize;

fn main() {
    // ── Detect the current override ──────────────────────────────────────
    let detected = detect_color_env();

    println!("Color environment detection result: {:?}", detected);
    println!();

    match detected {
        ColorEnvOverride::NoColor => {
            println!("  Color is DISABLED by an environment variable.");
            println!("  (NO_COLOR, FORCE_COLOR=0, or CLICOLOR=0 is set)");
        }
        ColorEnvOverride::ForceColor => {
            println!("  Color is FORCED ON (standard/256).");
            println!("  (FORCE_COLOR=1/2 or CLICOLOR_FORCE is set)");
        }
        ColorEnvOverride::ForceColorTruecolor => {
            println!("  Color is FORCED ON (truecolor/24-bit).");
            println!("  (FORCE_COLOR=3 is set)");
        }
        ColorEnvOverride::None => {
            println!("  No color override detected.");
            println!("  Normal terminal auto-detection will be used.");
        }
    }

    println!();

    // ── Show styled output via Console ───────────────────────────────────
    // The Console respects env vars automatically, so if NO_COLOR is set,
    // the styled output below will appear without ANSI codes.

    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .build();

    console.print(&Rule::with_title("Styled Output"));

    console.print(&"This text should be bold and red".bold().red());
    console.print(&"This text should be green".green());
    console.print(&"This text should be blue with italic".blue().italic());

    console.print(&Rule::with_title("Try It Yourself"));

    // Print instructions for testing
    let instructions = [
        "Run with different environment variables:",
        "",
        "  cargo run --example color_env           # auto-detect",
        "  NO_COLOR=1 cargo run --example color_env      # disable color",
        "  FORCE_COLOR=3 cargo run --example color_env   # force truecolor",
        "  FORCE_COLOR=0 cargo run --example color_env   # force off",
        "  CLICOLOR=0 cargo run --example color_env      # disable via CLICOLOR",
        "  CLICOLOR_FORCE=1 cargo run --example color_env # force on",
    ];

    for line in &instructions {
        console.print_text(line);
    }
}
