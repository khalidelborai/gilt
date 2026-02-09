//! Columns widget demonstration with generic Renderables
//!
//! This example shows how the Columns widget works with various Renderable types:
//! - Text items
//! - Styled strings  
//! - Panels (converted to strings)
//! - Tables (converted to strings)
//!
//! Run: cargo run --example columns_demo

use gilt::prelude::*;
use gilt::box_chars::{ASCII, DOUBLE, HEAVY, ROUNDED, SQUARE};
use gilt::panel::Panel;
use gilt::table::Table;
use gilt::text::JustifyMethod;

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // =======================================================================
    // 1. BASIC COLUMNS WITH TEXT ITEMS
    // =======================================================================
    console.rule(Some("1. Basic Columns - Text Items"));
    console.print_text("Auto-fitting columns with plain text items:\n");

    let languages = vec![
        "Rust", "Python", "Go", "TypeScript", "Java",
        "C++", "Ruby", "Swift", "Kotlin", "Haskell",
        "Elixir", "Zig", "C#", "PHP", "Lua",
    ];

    let mut cols = Columns::new();
    for lang in &languages {
        cols.add_renderable(lang);
    }
    console.print(&cols);

    // =======================================================================
    // 2. COLUMNS WITH STYLED STRINGS
    // =======================================================================
    console.rule(Some("2. Columns with Styled Strings"));
    console.print_text("Using markup for styled text in columns:\n");

    let styled_items = vec![
        "[bold red]Error[/bold red]",
        "[bold yellow]Warning[/bold yellow]",
        "[bold green]Success[/bold green]",
        "[bold blue]Info[/bold blue]",
        "[dim]Debug[/dim]",
        "[italic]Trace[/italic]",
    ];

    let mut cols = Columns::new();
    for item in &styled_items {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // =======================================================================
    // 3. EQUAL WIDTH COLUMNS
    // =======================================================================
    console.rule(Some("3. Equal-Width Columns"));
    console.print_text("Items distributed in equal-width columns:\n");

    let mut cols = Columns::new()
        .with_equal(true)
        .with_expand(true);
    
    for lang in &languages {
        cols.add_renderable(lang);
    }
    console.print(&cols);

    // =======================================================================
    // 4. COLUMNS WITH PANELS (Renderable → String)
    // =======================================================================
    console.rule(Some("4. Columns with Panels"));
    console.print_text("Panels rendered as columns (captured output):\n");

    // Create panels and render them to strings, then add to columns
    let panel_data = vec![
        ("Rust", "Memory safety without GC", "cyan"),
        ("Go", "Simple concurrency", "blue"),
        ("Python", "Batteries included", "yellow"),
        ("Zig", "Comptime power", "yellow"),
    ];

    let mut panel_strings: Vec<String> = Vec::new();
    for (name, desc, color) in panel_data {
        let panel = Panel::fit(Text::from_markup(desc).unwrap())
            .with_title(name)
            .with_border_style(Style::parse(color).unwrap());
        
        // Capture panel output as string
        let mut temp_console = Console::builder()
            .width(30)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&panel);
        panel_strings.push(temp_console.end_capture());
    }

    let mut cols = Columns::new()
        .with_padding((1, 2, 1, 2));
    
    for panel_str in &panel_strings {
        cols.add_renderable(panel_str.trim());
    }
    console.print(&cols);

    // =======================================================================
    // 5. COLUMNS WITH TABLES (Renderable → String)
    // =======================================================================
    console.rule(Some("5. Columns with Mini Tables"));
    console.print_text("Tables rendered side-by-side in columns:\n");

    let mut table_strings: Vec<String> = Vec::new();

    // Create small stat tables
    for (title, rows) in vec![
        ("CPU", vec![("User", "45%"), ("System", "12%"), ("Idle", "43%")]),
        ("Memory", vec![("Used", "8.2 GB"), ("Free", "7.8 GB"), ("Cache", "2.1 GB")]),
        ("Disk", vec![("Read", "120 MB/s"), ("Write", "85 MB/s"), ("IOPS", "2.4k")]),
    ] {
        let mut table = Table::grid(&["Metric", "Value"]);
        for (k, v) in rows {
            table.add_row(&[k, v]);
        }
        
        let mut temp_console = Console::builder()
            .width(25)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&table);
        
        // Add title above the table
        let output = format!("[bold]{}[/bold]\n{}", title, temp_console.end_capture());
        table_strings.push(output);
    }

    let mut cols = Columns::new()
        .with_padding((0, 3, 0, 3));
    
    for table_str in &table_strings {
        cols.add_renderable(table_str);
    }
    console.print(&cols);

    // =======================================================================
    // 6. COLUMN ALIGNMENT OPTIONS
    // =======================================================================
    console.rule(Some("6. Column Alignment Options"));
    console.print_text("Different alignments within columns:\n");

    // Left alignment (default)
    console.print_text("[dim]Left (default):[/dim]");
    let mut cols = Columns::new().with_width(15);
    for item in &["Short", "Medium length", "A much longer item here"] {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // Center alignment
    console.print_text("\n[dim]Center:[/dim]");
    let mut cols = Columns::new()
        .with_width(15)
        .with_align(JustifyMethod::Center);
    for item in &["Short", "Medium length", "A much longer item here"] {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // Right alignment
    console.print_text("\n[dim]Right:[/dim]");
    let mut cols = Columns::new()
        .with_width(15)
        .with_align(JustifyMethod::Right);
    for item in &["Short", "Medium length", "A much longer item here"] {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // =======================================================================
    // 7. DYNAMIC COLUMN COUNT BASED ON TERMINAL WIDTH
    // =======================================================================
    console.rule(Some("7. Dynamic Column Count"));
    console.print_text("Auto-fitting based on terminal width (resize to see effect):\n");

    let items: Vec<String> = (1..=20)
        .map(|i| format!("Item {:02}", i))
        .collect();

    // Simulate different terminal widths
    for (width, label) in vec![(120, "Wide (120 cols)"), (80, "Medium (80 cols)"), (50, "Narrow (50 cols)")] {
        console.print_text(&format!("\n[dim]{}:[/dim]", label));
        let mut cols = Columns::new();
        for item in &items {
            cols.add_renderable(item);
        }
        
        // Use a console with specific width to demonstrate
        let mut temp_console = Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&cols);
        console.print_text(&temp_console.end_capture());
    }

    // =======================================================================
    // 8. PRACTICAL: FILE BROWSER STYLE LAYOUT
    // =======================================================================
    console.rule(Some("8. File Browser Layout"));
    console.print_text("Mimicking a file manager with icons and metadata:\n");

    let files = vec![
        ("📄", "README.md", "2.1 KB", "Jan 15"),
        ("📁", "src", "--", "Jan 14"),
        ("📄", "Cargo.toml", "1.2 KB", "Jan 13"),
        ("📁", "tests", "--", "Jan 12"),
        ("📄", "LICENSE", "1.0 KB", "Jan 10"),
        ("📄", "CHANGELOG.md", "4.5 KB", "Jan 08"),
        ("📁", "examples", "--", "Jan 05"),
        ("📄", "justfile", "0.5 KB", "Jan 03"),
    ];

    // Create styled file entries
    let mut file_strings: Vec<String> = Vec::new();
    for (icon, name, size, date) in files {
        let entry = format!(
            "{} [bold]{}[/bold]\n    [dim]{}  {}[/dim]",
            icon, name, size, date
        );
        file_strings.push(entry);
    }

    let mut cols = Columns::new()
        .with_padding((0, 2, 1, 2));
    
    for file_str in &file_strings {
        cols.add_renderable(file_str);
    }
    console.print(&cols);

    // =======================================================================
    // 9. PRACTICAL: DASHBOARD CARDS
    // =======================================================================
    console.rule(Some("9. Dashboard Cards"));
    console.print_text("System dashboard with metric cards:\n");

    let dashboard_cards = vec![
        ("🚀", "Requests/sec", "1,234", "+12%", "green"),
        ("⏱️", "Avg Latency", "45ms", "-5%", "green"),
        ("❌", "Error Rate", "0.02%", "+0.01%", "red"),
        ("👥", "Active Users", "5,678", "+89", "green"),
        ("💾", "DB Connections", "42/100", "stable", "blue"),
        ("🔄", "Cache Hit Rate", "94.5%", "+2.1%", "green"),
    ];

    let mut card_strings: Vec<String> = Vec::new();
    for (icon, label, value, change, color) in dashboard_cards {
        let card = Panel::fit(
            Text::from_markup(&format!(
                "{}\n[bold {}]{}[/bold {}]\n[dim]{}[/dim]",
                label, color, value, color, change
            )).unwrap()
        )
        .with_title(format!("{} {}", icon, label))
        .with_box_chars(&ROUNDED)
        .with_border_style(Style::parse(color).unwrap());

        let mut temp_console = Console::builder()
            .width(25)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&card);
        card_strings.push(temp_console.end_capture());
    }

    let mut cols = Columns::new()
        .with_padding((1, 2, 1, 2));
    
    for card_str in &card_strings {
        cols.add_renderable(card_str.trim());
    }
    console.print(&cols);

    // =======================================================================
    // 10. PRACTICAL: GALLERY VIEW
    // =======================================================================
    console.rule(Some("10. Gallery View"));
    console.print_text("Image gallery with captions:\n");

    let gallery_items = vec![
        ("[yellow]█[/yellow][green]▄[/green]", "Landscape", "1920x1080"),
        ("[blue]▓[/blue][cyan]▒[/cyan]░", "Portrait", "1080x1920"),
        ("[red]▪[/red][magenta]◆[/magenta]", "Abstract", "800x600"),
        ("[white]▓[/white][dim]█[/dim]", "B&W Photo", "2048x1536"),
        ("[green]░[/green][yellow]▒[/yellow]", "Nature", "2560x1440"),
        ("[cyan]▓[/cyan][blue]█[/blue]", "Ocean", "3840x2160"),
    ];

    let mut gallery_strings: Vec<String> = Vec::new();
    for (preview, title, dims) in gallery_items {
        // Create a "thumbnail" using box drawing
        let thumbnail = format!(
            "[on black]{:^10}[/on black]\n  [bold]{}[/bold]\n  [dim]{}[/dim]",
            preview, title, dims
        );
        gallery_strings.push(thumbnail);
    }

    let mut cols = Columns::new()
        .with_width(12)
        .with_align(JustifyMethod::Center)
        .with_padding((1, 2, 1, 2));
    
    for item in &gallery_strings {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // =======================================================================
    // 11. COLUMN-FIRST ORDERING
    // =======================================================================
    console.rule(Some("11. Column-First Ordering"));
    console.print_text("Items fill top-to-bottom first, then left-to-right:\n");

    let numbered: Vec<String> = (1..=12)
        .map(|i| format!("Item {}", i))
        .collect();

    let mut cols = Columns::new()
        .with_column_first(true)
        .with_width(10);
    
    for item in &numbered {
        cols.add_renderable(item);
    }
    console.print(&cols);

    // =======================================================================
    // 12. DIFFERENT BOX STYLES IN COLUMNS
    // =======================================================================
    console.rule(Some("12. Panel Variations in Columns"));
    console.print_text("Different border styles for each panel:\n");

    let box_styles = vec![
        (&ROUNDED, "Rounded", "cyan"),
        (&SQUARE, "Square", "yellow"),
        (&DOUBLE, "Double", "magenta"),
        (&HEAVY, "Heavy", "red"),
        (&ASCII, "ASCII", "green"),
    ];

    let mut style_strings: Vec<String> = Vec::new();
    for (chars, name, color) in box_styles {
        let panel = Panel::fit(Text::new(name, Style::null()))
            .with_box_chars(chars)
            .with_border_style(Style::parse(color).unwrap());

        let mut temp_console = Console::builder()
            .width(15)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&panel);
        style_strings.push(temp_console.end_capture());
    }

    let mut cols = Columns::new()
        .with_padding((1, 1, 1, 1));
    
    for style_str in &style_strings {
        cols.add_renderable(style_str.trim());
    }
    console.print(&cols);

    // =======================================================================
    // 13. TITLE AND PADDING CONFIGURATION
    // =======================================================================
    console.rule(Some("13. Columns with Title"));
    console.print_text("Columns widget with a title above:\n");

    let mut cols = Columns::new()
        .with_title("Programming Paradigms")
        .with_equal(true)
        .with_expand(true)
        .with_padding((1, 2, 1, 2));

    let paradigms = vec![
        "Object-Oriented",
        "Functional",
        "Procedural", 
        "Declarative",
        "Concurrent",
        "Logic",
    ];
    
    for p in &paradigms {
        cols.add_renderable(p);
    }
    console.print(&cols);

    // =======================================================================
    // 14. NESTED STRUCTURES DEMONSTRATION
    // =======================================================================
    console.rule(Some("14. Complex Nested Renderables"));
    console.print_text("Combining multiple renderable types:\n");

    // Create a complex layout with tables inside panels
    let mut complex_strings: Vec<String> = Vec::new();

    for section in vec!["Frontend", "Backend", "Database"] {
        let mut table = Table::grid(&["Tech", "Status"]);
        let rows = match section {
            "Frontend" => vec![("React", "✓"), ("TypeScript", "✓"), ("Vite", "○")],
            "Backend" => vec![("Rust", "✓"), ("Actix", "✓"), ("Redis", "○")],
            "Database" => vec![("PostgreSQL", "✓"), ("TimescaleDB", "✓")],
            _ => vec![],
        };
        
        for (tech, status) in rows {
            table.add_row(&[tech, status]);
        }

        let panel = Panel::new(Text::from_markup(&table.to_string()).unwrap())
            .with_title(section)
            .with_box_chars(&ROUNDED)
            .with_expand(false);

        let mut temp_console = Console::builder()
            .width(25)
            .force_terminal(true)
            .no_color(false)
            .build();
        temp_console.begin_capture();
        temp_console.print(&panel);
        complex_strings.push(temp_console.end_capture());
    }

    let mut cols = Columns::new()
        .with_padding((0, 2, 0, 2));
    
    for complex_str in &complex_strings {
        cols.add_renderable(complex_str.trim());
    }
    console.print(&cols);

    // =======================================================================
    // END
    // =======================================================================
    console.rule(Some("End of Columns Demo"));
    console.print_text("\n[dim]Try resizing your terminal and running again![/dim]");
}
