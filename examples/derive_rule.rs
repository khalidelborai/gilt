//! Demonstrates gilt's #[derive(Rule)] proc macro.
//! Run with: cargo run --example derive_rule --features derive

#[cfg(feature = "derive")]
fn main() {
    use gilt::prelude::*;
    use gilt::DeriveRule;

    let mut console = Console::new();

    // ── 1. Basic derive — struct name becomes the title ─────────────────
    console.print_text("\n[bold cyan]1. Basic derive — struct name as title (no attributes)[/]");

    #[derive(DeriveRule)]
    struct Introduction {
        _placeholder: u8,
    }

    let intro = Introduction { _placeholder: 0 };
    console.print(&intro.to_rule());

    // ── 2. Custom style and characters ──────────────────────────────────
    console.print_text("\n[bold cyan]2. Custom style and characters[/]");

    #[derive(DeriveRule)]
    #[rule(title = "Configuration", characters = "═", style = "bold magenta")]
    struct ConfigSection {
        _placeholder: u8,
    }

    let config = ConfigSection { _placeholder: 0 };
    console.print(&config.to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Warnings", characters = "!", style = "bold yellow")]
    struct WarningSection {
        _placeholder: u8,
    }

    let warn = WarningSection { _placeholder: 0 };
    console.print(&warn.to_rule());

    // ── 3. Field-driven title — #[rule(title)] on a field ───────────────
    console.print_text("\n[bold cyan]3. Field-driven title — title from a struct field[/]");

    #[derive(DeriveRule)]
    #[rule(characters = "─", style = "bold green")]
    struct Chapter {
        #[rule(title)]
        name: String,
        _number: u32,
    }

    let ch1 = Chapter {
        name: "Getting Started".into(),
        _number: 1,
    };
    console.print(&ch1.to_rule());

    let ch2 = Chapter {
        name: "Advanced Patterns".into(),
        _number: 2,
    };
    console.print(&ch2.to_rule());

    let ch3 = Chapter {
        name: "Conclusion".into(),
        _number: 3,
    };
    console.print(&ch3.to_rule());

    // ── 4. Alignment options ────────────────────────────────────────────
    console.print_text("\n[bold cyan]4. Alignment options — left, center, right[/]");

    #[derive(DeriveRule)]
    #[rule(title = "Left Aligned", align = "left", style = "blue")]
    struct LeftRule {
        _placeholder: u8,
    }

    let left = LeftRule { _placeholder: 0 };
    console.print(&left.to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Center Aligned", align = "center", style = "cyan")]
    struct CenterRule {
        _placeholder: u8,
    }

    let center = CenterRule { _placeholder: 0 };
    console.print(&center.to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Right Aligned", align = "right", style = "green")]
    struct RightRule {
        _placeholder: u8,
    }

    let right = RightRule { _placeholder: 0 };
    console.print(&right.to_rule());

    // ── 5. Section dividers showcase ────────────────────────────────────
    console.print_text("\n[bold cyan]5. Section dividers — different characters for visual variety[/]");

    #[derive(DeriveRule)]
    #[rule(title = "Heavy", characters = "━", style = "bold red")]
    struct HeavyDivider {
        _placeholder: u8,
    }
    console.print(&(HeavyDivider { _placeholder: 0 }).to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Double", characters = "═", style = "bold bright_blue")]
    struct DoubleDivider {
        _placeholder: u8,
    }
    console.print(&(DoubleDivider { _placeholder: 0 }).to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Dashed", characters = "╌", style = "dim white")]
    struct DashedDivider {
        _placeholder: u8,
    }
    console.print(&(DashedDivider { _placeholder: 0 }).to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Stars", characters = "★", style = "bold yellow")]
    struct StarDivider {
        _placeholder: u8,
    }
    console.print(&(StarDivider { _placeholder: 0 }).to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Dots", characters = "·", style = "bright_magenta")]
    struct DotDivider {
        _placeholder: u8,
    }
    console.print(&(DotDivider { _placeholder: 0 }).to_rule());

    #[derive(DeriveRule)]
    #[rule(title = "Waves", characters = "~", style = "bright_cyan")]
    struct WaveDivider {
        _placeholder: u8,
    }
    console.print(&(WaveDivider { _placeholder: 0 }).to_rule());
}

#[cfg(not(feature = "derive"))]
fn main() {
    eprintln!(
        "This example requires the 'derive' feature: cargo run --example derive_rule --features derive"
    );
}
