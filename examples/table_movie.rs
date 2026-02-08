//! Animated-style table construction demo — inspired by Python rich's table_movie.py.
//!
//! Builds a Star Wars table step by step, printing each stage with a Rule
//! separator.  Rather than using Live (which requires threading), this example
//! prints the table at each stage of construction so the reader can see how
//! columns, rows, and styling are layered on incrementally.

use gilt::box_chars::{ROUNDED, SIMPLE, SIMPLE_HEAVY};
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::table::{ColumnOptions, Table};
use gilt::text::JustifyMethod;

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Stage 1: Empty table with just column headers ──────────────────────
    console.print(&Rule::with_title("Stage 1 — Columns Only"));

    let table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    console.print(&table);

    // ── Stage 2: Add a couple of rows ──────────────────────────────────────
    console.print(&Rule::with_title("Stage 2 — First Rows"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    table.add_row(&["IV", "A New Hope", "George Lucas", "1977", "$775M"]);
    table.add_row(&[
        "V",
        "The Empire Strikes Back",
        "Irvin Kershner",
        "1980",
        "$547M",
    ]);
    console.print(&table);

    // ── Stage 3: All rows ──────────────────────────────────────────────────
    console.print(&Rule::with_title("Stage 3 — All Rows"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    add_all_rows(&mut table);
    console.print(&table);

    // ── Stage 4: Title and caption ─────────────────────────────────────────
    console.print(&Rule::with_title("Stage 4 — Title & Caption"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    add_all_rows(&mut table);
    table.title = Some("Star Wars Saga".to_string());
    table.caption = Some("Source: Box Office Mojo".to_string());
    console.print(&table);

    // ── Stage 5: Style the border ──────────────────────────────────────────
    console.print(&Rule::with_title("Stage 5 — Rounded Border + Style"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    add_all_rows(&mut table);
    table.title = Some("Star Wars Saga".to_string());
    table.caption = Some("Source: Box Office Mojo".to_string());
    table.box_chars = Some(&ROUNDED);
    table.border_style = "bright_cyan".to_string();
    table.title_style = "bold white".to_string();
    table.caption_style = "dim".to_string();
    console.print(&table);

    // ── Stage 6: Row styles (alternating) ──────────────────────────────────
    console.print(&Rule::with_title("Stage 6 — Alternating Row Styles"));

    let mut table = Table::new(&["Episode", "Title", "Director", "Year", "Box Office"]);
    add_all_rows(&mut table);
    table.title = Some("Star Wars Saga".to_string());
    table.caption = Some("Source: Box Office Mojo".to_string());
    table.box_chars = Some(&ROUNDED);
    table.border_style = "bright_cyan".to_string();
    table.title_style = "bold white".to_string();
    table.caption_style = "dim".to_string();
    table.row_styles = vec!["".to_string(), "dim".to_string()];
    console.print(&table);

    // ── Stage 7: Column-level styling ──────────────────────────────────────
    console.print(&Rule::with_title("Stage 7 — Column Justification"));

    let mut table = Table::new(&[]);
    table.add_column(
        "Episode",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Center),
            ..Default::default()
        },
    );
    table.add_column(
        "Title",
        "",
        ColumnOptions {
            style: Some("bold".to_string()),
            ..Default::default()
        },
    );
    table.add_column("Director", "", Default::default());
    table.add_column(
        "Year",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Center),
            ..Default::default()
        },
    );
    table.add_column(
        "Box Office",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Right),
            style: Some("green".to_string()),
            ..Default::default()
        },
    );
    add_all_rows(&mut table);
    table.title = Some("Star Wars Saga".to_string());
    table.caption = Some("Source: Box Office Mojo".to_string());
    table.box_chars = Some(&ROUNDED);
    table.border_style = "bright_cyan".to_string();
    table.title_style = "bold white".to_string();
    table.caption_style = "dim".to_string();
    table.row_styles = vec!["".to_string(), "dim".to_string()];
    console.print(&table);

    // ── Stage 8: Different box styles ──────────────────────────────────────
    console.print(&Rule::with_title("Stage 8a — Simple Box"));

    let mut table = build_final_table();
    table.box_chars = Some(&SIMPLE);
    console.print(&table);

    console.print(&Rule::with_title("Stage 8b — Simple Heavy Box"));

    let mut table = build_final_table();
    table.box_chars = Some(&SIMPLE_HEAVY);
    console.print(&table);

    // ── Final: expanded table ──────────────────────────────────────────────
    console.print(&Rule::with_title("Final — Expanded to Full Width"));

    let mut table = build_final_table();
    table.set_expand(true);
    console.print(&table);
}

/// Helper: add all Star Wars movie rows to a table.
fn add_all_rows(table: &mut Table) {
    table.add_row(&["IV", "A New Hope", "George Lucas", "1977", "$775M"]);
    table.add_row(&[
        "V",
        "The Empire Strikes Back",
        "Irvin Kershner",
        "1980",
        "$547M",
    ]);
    table.add_row(&[
        "VI",
        "Return of the Jedi",
        "Richard Marquand",
        "1983",
        "$475M",
    ]);
    table.add_row(&["I", "The Phantom Menace", "George Lucas", "1999", "$1.03B"]);
    table.add_row(&[
        "II",
        "Attack of the Clones",
        "George Lucas",
        "2002",
        "$653M",
    ]);
    table.add_row(&[
        "III",
        "Revenge of the Sith",
        "George Lucas",
        "2005",
        "$868M",
    ]);
    table.add_row(&["VII", "The Force Awakens", "J.J. Abrams", "2015", "$2.07B"]);
    table.add_row(&["VIII", "The Last Jedi", "Rian Johnson", "2017", "$1.33B"]);
    table.add_row(&[
        "IX",
        "The Rise of Skywalker",
        "J.J. Abrams",
        "2019",
        "$1.07B",
    ]);
}

/// Helper: build the fully-styled table used for the final stages.
fn build_final_table() -> Table {
    let mut table = Table::new(&[]);
    table.add_column(
        "Episode",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Center),
            ..Default::default()
        },
    );
    table.add_column(
        "Title",
        "",
        ColumnOptions {
            style: Some("bold".to_string()),
            ..Default::default()
        },
    );
    table.add_column("Director", "", Default::default());
    table.add_column(
        "Year",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Center),
            ..Default::default()
        },
    );
    table.add_column(
        "Box Office",
        "",
        ColumnOptions {
            justify: Some(JustifyMethod::Right),
            style: Some("green".to_string()),
            ..Default::default()
        },
    );
    add_all_rows(&mut table);
    table.title = Some("Star Wars Saga".to_string());
    table.caption = Some("Source: Box Office Mojo".to_string());
    table.box_chars = Some(&ROUNDED);
    table.border_style = "bright_cyan".to_string();
    table.title_style = "bold white".to_string();
    table.caption_style = "dim".to_string();
    table.row_styles = vec!["".to_string(), "dim".to_string()];
    table
}
