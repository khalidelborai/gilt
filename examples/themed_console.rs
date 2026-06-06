//! Theme system — write a custom JSON theme and resolve style names through it.
//!
//! Demonstrates:
//! - `Theme::from_json_str(json)` — parse a named-style map from JSON.
//! - `ConsoleBuilder::theme_from_path(path)` — auto-load at construction time.
//! - `console.get_style("name")` — look up a named style from the stack.
//! - `console.push_theme(theme)` / `console.pop_theme()` — dynamic overrides.
//! - The `GILT_THEME` environment variable (documented below).
//!
//! Environment variable (not demonstrated live here, documented only):
//!   `GILT_THEME=/path/to/theme.json` — loaded at Console::new() time when the
//!   `json` feature is enabled and the build is native (non-wasm).
//!
//! Run with: cargo run --example themed_console --features json

#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
fn main() {
    use gilt::console::Console;
    use gilt::theme::Theme;

    // ---- 1. Build a Theme from a JSON string --------------------------------

    let json = r#"{
        "brand.primary":   "bold #0078D4",
        "brand.success":   "bold green",
        "brand.warning":   "bold yellow",
        "brand.error":     "bold red",
        "brand.muted":     "dim grey50"
    }"#;

    let theme = Theme::from_json_str(json).expect("valid JSON theme");
    println!("Loaded theme with {} custom styles", theme.styles.len());

    // ---- 2. Push the theme onto a console -----------------------------------

    let mut console = Console::builder().width(80).force_terminal(true).build();
    console.push_theme(theme);

    // Resolve a named style through the active theme stack.
    let primary = console
        .get_style("brand.primary")
        .expect("brand.primary resolves");
    println!("brand.primary = {}", primary);

    // Print with the resolved style by embedding it in markup.
    console.print_text("[brand.primary]gilt themed output[/]");
    console.print_text("[brand.success]  success[/]  build passed");
    console.print_text("[brand.warning]  warning[/]  deprecated API");
    console.print_text("[brand.error]    error[/]    file not found");
    console.print_text("[brand.muted]    muted  [/]  verbose detail");

    // Pop the theme to return to defaults.
    console.pop_theme();

    println!("\n(theme popped — 'brand.primary' now falls back to a style parse)");

    // ---- 3. theme_from_path: write the same JSON to a temp file and reload -

    let path = std::env::temp_dir().join("gilt_example_theme.json");
    std::fs::write(&path, json).unwrap();

    let mut console2 = Console::builder()
        .width(80)
        .force_terminal(true)
        .theme_from_path(&path)
        .build();

    let primary2 = console2
        .get_style("brand.primary")
        .expect("brand.primary loaded from file");
    println!(
        "\nbrand.primary (loaded from {:?}) = {}",
        path.file_name().unwrap(),
        primary2
    );
    console2.print_text("[brand.primary]theme loaded from file![/]");

    // Clean up.
    let _ = std::fs::remove_file(&path);

    println!("\n[tip] Set GILT_THEME=/path/to/theme.json to auto-load at Console::new()");
}

#[cfg(not(all(feature = "json", not(target_arch = "wasm32"))))]
fn main() {
    eprintln!("This example requires --features json on a non-wasm target");
}
