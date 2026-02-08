//! Demonstrates gilt's color system — parsing, downgrading, and ANSI code generation.

use gilt::color::{blend_rgb, Color, ColorSystem};
use gilt::color_triplet::ColorTriplet;

fn main() {
    println!("=== Color Parsing ===\n");

    let colors = ["red", "bright_cyan", "#ff6600", "color(100)", "rgb(50,150,250)", "default"];
    for name in colors {
        let color = Color::parse(name).unwrap();
        println!(
            "{:<20} type={:<10?} number={:<6} system={:?}",
            name,
            color.color_type,
            color.number.map_or("-".into(), |n| n.to_string()),
            color.system()
        );
    }

    println!("\n=== ANSI Escape Codes ===\n");

    let pairs = [("red", true), ("red", false), ("#ff0000", true), ("color(100)", true)];
    for (name, fg) in pairs {
        let color = Color::parse(name).unwrap();
        let codes = color.get_ansi_codes(fg);
        println!(
            "{:<15} {} => \\e[{}m",
            name,
            if fg { "fg" } else { "bg" },
            codes.join(";")
        );
    }

    println!("\n=== Color Downgrading ===\n");

    let truecolor = Color::parse("#ff6347").unwrap(); // tomato
    println!("Original:  {:?}", truecolor);
    println!("→ 256-color: {:?}", truecolor.downgrade(ColorSystem::EightBit));
    println!("→ 16-color:  {:?}", truecolor.downgrade(ColorSystem::Standard));
    println!("→ Windows:   {:?}", truecolor.downgrade(ColorSystem::Windows));

    println!("\n=== Color Blending ===\n");

    let red = ColorTriplet::new(255, 0, 0);
    let blue = ColorTriplet::new(0, 0, 255);
    for pct in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let blended = blend_rgb(red, blue, pct);
        println!("  red →{:>3.0}%→ blue = {}", pct * 100.0, blended.hex());
    }
}
