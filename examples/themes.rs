//! Demonstrates gilt's built-in color themes.
//!
//! This example showcases all available pre-defined themes with their
//! color palettes and sample output.

use gilt::color::Color;
use gilt::style::Style;
use gilt::theme::{
    self, dracula, github_dark, github_light, monokai, nord, one_dark, solarized_dark,
    solarized_light, Theme,
};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Gilt Built-in Color Themes Demo                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // List all available themes
    println!("📋 Available Built-in Themes:");
    println!("   ──────────────────────────────────────────────────────────────");
    for name in theme::builtin_themes() {
        println!("   • {}", name);
    }
    println!();

    // Demonstrate each theme
    print_theme(
        "Monokai",
        Theme::monokai(),
        &[
            ("Background", monokai::BG),
            ("Foreground", monokai::FG),
            ("Red", monokai::RED),
            ("Green", monokai::GREEN),
            ("Yellow", monokai::YELLOW),
            ("Blue", monokai::BLUE),
            ("Purple", monokai::PURPLE),
            ("Cyan", monokai::CYAN),
            ("Orange", monokai::ORANGE),
        ],
    );

    print_theme(
        "Solarized Dark",
        Theme::solarized_dark(),
        &[
            ("Background", solarized_dark::BG),
            ("Foreground", solarized_dark::FG),
            ("Yellow", solarized_dark::YELLOW),
            ("Orange", solarized_dark::ORANGE),
            ("Red", solarized_dark::RED),
            ("Magenta", solarized_dark::MAGENTA),
            ("Violet", solarized_dark::VIOLET),
            ("Blue", solarized_dark::BLUE),
            ("Cyan", solarized_dark::CYAN),
            ("Green", solarized_dark::GREEN),
        ],
    );

    print_theme(
        "Solarized Light",
        Theme::solarized_light(),
        &[
            ("Background", solarized_light::BG),
            ("Foreground", solarized_light::FG),
            ("Yellow", solarized_light::YELLOW),
            ("Orange", solarized_light::ORANGE),
            ("Red", solarized_light::RED),
            ("Magenta", solarized_light::MAGENTA),
            ("Violet", solarized_light::VIOLET),
            ("Blue", solarized_light::BLUE),
            ("Cyan", solarized_light::CYAN),
            ("Green", solarized_light::GREEN),
        ],
    );

    print_theme(
        "Dracula",
        Theme::dracula(),
        &[
            ("Background", dracula::BG),
            ("Foreground", dracula::FG),
            ("Red", dracula::RED),
            ("Green", dracula::GREEN),
            ("Yellow", dracula::YELLOW),
            ("Purple", dracula::PURPLE),
            ("Pink", dracula::PINK),
            ("Cyan", dracula::CYAN),
            ("Orange", dracula::ORANGE),
        ],
    );

    print_theme(
        "Nord",
        Theme::nord(),
        &[
            ("Background", nord::BG),
            ("Foreground", nord::FG),
            ("Frost 1", nord::FROST_1),
            ("Frost 2", nord::FROST_2),
            ("Frost 3", nord::FROST_3),
            ("Frost 4", nord::FROST_4),
            ("Aurora Red", nord::RED),
            ("Aurora Orange", nord::ORANGE),
            ("Aurora Yellow", nord::YELLOW),
            ("Aurora Green", nord::GREEN),
            ("Aurora Purple", nord::PURPLE),
        ],
    );

    print_theme(
        "One Dark",
        Theme::one_dark(),
        &[
            ("Background", one_dark::BG),
            ("Foreground", one_dark::FG),
            ("Red", one_dark::RED),
            ("Green", one_dark::GREEN),
            ("Yellow", one_dark::YELLOW),
            ("Blue", one_dark::BLUE),
            ("Magenta", one_dark::MAGENTA),
            ("Cyan", one_dark::CYAN),
        ],
    );

    print_theme(
        "GitHub Dark",
        Theme::github_dark(),
        &[
            ("Background", github_dark::BG),
            ("Foreground", github_dark::FG),
            ("Blue", github_dark::BLUE),
            ("Green", github_dark::GREEN),
            ("Yellow", github_dark::YELLOW),
            ("Orange", github_dark::ORANGE),
            ("Red", github_dark::RED),
            ("Purple", github_dark::PURPLE),
        ],
    );

    print_theme(
        "GitHub Light",
        Theme::github_light(),
        &[
            ("Background", github_light::BG),
            ("Foreground", github_light::FG),
            ("Blue", github_light::BLUE),
            ("Green", github_light::GREEN),
            ("Yellow", github_light::YELLOW),
            ("Orange", github_light::ORANGE),
            ("Red", github_light::RED),
            ("Purple", github_light::PURPLE),
        ],
    );

    // Demonstrate theme lookup
    println!("🔍 Theme Lookup Demo:");
    println!("   ──────────────────────────────────────────────────────────────");
    if let Some(theme) = theme::get_builtin_theme("monokai") {
        println!("   ✓ Successfully loaded 'monokai' theme");
        println!("     Has 'dim' style: {}", theme.get("dim").is_some());
    }
    if theme::get_builtin_theme("nonexistent").is_none() {
        println!("   ✓ 'nonexistent' theme correctly returned None");
    }
    println!();

    // Show how to use theme colors in styles
    println!("🎨 Using Theme Colors in Styles:");
    println!("   ──────────────────────────────────────────────────────────────");
    let red = Style::parse(&format!("bold {}", monokai::RED)).unwrap();
    let green = Style::parse(&format!("{}", monokai::GREEN)).unwrap();
    let blue = Style::parse(&format!("{}", monokai::BLUE)).unwrap();

    println!(
        "   Red:   {} {}",
        red.render("████", Some(gilt::color::ColorSystem::TrueColor)),
        monokai::RED
    );
    println!(
        "   Green: {} {}",
        green.render("████", Some(gilt::color::ColorSystem::TrueColor)),
        monokai::GREEN
    );
    println!(
        "   Blue:  {} {}",
        blue.render("████", Some(gilt::color::ColorSystem::TrueColor)),
        monokai::BLUE
    );
    println!();

    // Show theme color constants documentation
    println!("📚 Color Constants (available in theme module):");
    println!("   ──────────────────────────────────────────────────────────────");
    println!("   use gilt::theme::monokai;");
    println!("   let bg_color = Color::parse(monokai::BG).unwrap();");
    println!();

    let bg_color = Color::parse(monokai::BG).unwrap();
    println!("   monokai::BG = {} → {:?}", monokai::BG, bg_color);
    println!();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  All themes are ready to use!                                  ║");
    println!("║  Use Theme::monokai(), Theme::dracula(), etc.                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

fn print_theme(name: &str, _theme: Theme, colors: &[(&str, &str)]) {
    println!("┌─ Theme: {:56}─┐", format!("{}", name));
    println!("│{:64}│", "");

    for (color_name, hex) in colors {
        let style = Style::parse(hex).unwrap();
        let rendered = style.render("████", Some(gilt::color::ColorSystem::TrueColor));
        println!(
            "│  {:12} {} {:44}│",
            color_name,
            rendered,
            format!("({})", hex)
        );
    }

    println!("│{:64}│", "");
    println!(
        "└{:64}┘",
        "────────────────────────────────────────────────────────────────"
    );
    println!();
}
