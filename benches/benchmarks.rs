//! Criterion benchmarks for gilt's hot paths.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use gilt::color::{Color, ColorSystem};
use gilt::color_triplet::ColorTriplet;
use gilt::console::ConsoleBuilder;
use gilt::highlighter::{Highlighter, ReprHighlighter};
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Text;

// ---------------------------------------------------------------------------
// a) Style parsing
// ---------------------------------------------------------------------------

fn bench_style_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_parsing");

    group.bench_function("bold_red_on_blue", |b| {
        b.iter(|| Style::parse(black_box("bold red on blue")).unwrap());
    });

    group.bench_function("dim_italic_underline", |b| {
        b.iter(|| Style::parse(black_box("dim italic underline")).unwrap());
    });

    group.bench_function("complex_hex_colors", |b| {
        b.iter(|| {
            Style::parse(black_box("bold italic underline strike #ff5733 on #1a1a2e")).unwrap()
        });
    });

    group.bench_function("combine_three", |b| {
        let s1 = Style::parse("bold red").unwrap();
        let s2 = Style::parse("italic on blue").unwrap();
        let s3 = Style::parse("underline").unwrap();
        let styles = [s1, s2, s3];
        b.iter(|| Style::combine(black_box(&styles)));
    });

    group.bench_function("render_ansi", |b| {
        let style = Style::parse("bold red on blue").unwrap();
        b.iter(|| style.render(black_box("Hello, World!"), Some(ColorSystem::TrueColor)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// b) Text creation
// ---------------------------------------------------------------------------

fn bench_text_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_creation");

    group.bench_function("new_short", |b| {
        b.iter(|| Text::new(black_box("Hello, World!"), Style::null()));
    });

    let long = "The quick brown fox jumps over the lazy dog. ".repeat(20);
    group.bench_function("new_long", |b| {
        b.iter(|| Text::new(black_box(&*long), Style::null()));
    });

    group.bench_function("styled", |b| {
        let style = Style::parse("bold red").unwrap();
        b.iter(|| Text::styled(black_box("Hello, World!"), style.clone()));
    });

    group.bench_function("from_markup_simple", |b| {
        b.iter(|| Text::from_markup(black_box("[bold]Hello[/bold] World")).unwrap());
    });

    group.bench_function("from_markup_nested", |b| {
        b.iter(|| {
            Text::from_markup(black_box(
                "[bold red]Hello [italic]beautiful [underline]world[/underline][/italic][/bold red]",
            ))
            .unwrap()
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// c) Text operations
// ---------------------------------------------------------------------------

fn bench_text_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_operations");

    // cell_len
    let ascii_text = Text::new("Hello, World! This is a test string.", Style::null());
    group.bench_function("cell_len_ascii", |b| {
        b.iter(|| black_box(&ascii_text).cell_len());
    });

    let unicode_text = Text::new(
        "\u{1F600}\u{1F601}\u{1F602} CJK: \u{4E16}\u{754C}\u{4F60}\u{597D}",
        Style::null(),
    );
    group.bench_function("cell_len_unicode", |b| {
        b.iter(|| black_box(&unicode_text).cell_len());
    });

    // wrap
    let short_text = Text::new("Hello, World!", Style::null());
    group.bench_function("wrap_short", |b| {
        b.iter(|| short_text.wrap(black_box(80), None, None, 4, false));
    });

    let paragraph = "The quick brown fox jumps over the lazy dog. \
        Pack my box with five dozen liquor jugs. \
        How vexingly quick daft zebras jump. \
        The five boxing wizards jump quickly. \
        Sphinx of black quartz, judge my vow.";
    let para_text = Text::new(paragraph, Style::null());
    group.bench_function("wrap_paragraph", |b| {
        b.iter(|| para_text.wrap(black_box(40), None, None, 4, false));
    });

    let styled_para = Text::from_markup(
        "[bold]The quick brown fox[/bold] jumps over the [italic red]lazy dog[/italic red]. \
         [underline]Pack my box with five dozen liquor jugs.[/underline] \
         How [bold green]vexingly quick[/bold green] daft zebras jump.",
    )
    .unwrap();
    group.bench_function("wrap_styled_paragraph", |b| {
        b.iter(|| styled_para.wrap(black_box(40), None, None, 4, false));
    });

    // truncate
    group.bench_function("truncate", |b| {
        b.iter_batched(
            || {
                Text::new(
                    "Hello, World! This is a long string that should be truncated.",
                    Style::null(),
                )
            },
            |mut text| text.truncate(black_box(20), None, false),
            BatchSize::SmallInput,
        );
    });

    // split
    let newline_text = Text::new("Line 1\nLine 2\nLine 3\nLine 4\nLine 5", Style::null());
    group.bench_function("split_newlines", |b| {
        b.iter(|| newline_text.split(black_box("\n"), false, false));
    });

    let word_text = Text::new("The quick brown fox jumps over the lazy dog", Style::null());
    group.bench_function("split_words", |b| {
        b.iter(|| word_text.split(black_box(" "), false, false));
    });

    // divide
    let divide_text = Text::new("The quick brown fox jumps over the lazy dog", Style::null());
    group.bench_function("divide", |b| {
        b.iter(|| divide_text.divide(black_box(&[4, 10, 16, 20, 26, 31, 35, 40])));
    });

    // measure
    let measure_text = Text::new("Hello, World! This is a test.", Style::null());
    group.bench_function("measure", |b| {
        b.iter(|| black_box(&measure_text).measure());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// d) Color downgrade
// ---------------------------------------------------------------------------

fn bench_color_downgrade(c: &mut Criterion) {
    let mut group = c.benchmark_group("color_downgrade");

    let color_vivid = Color::from_triplet(ColorTriplet::new(171, 82, 219));
    group.bench_function("truecolor_to_256", |b| {
        b.iter(|| black_box(&color_vivid).downgrade(ColorSystem::EightBit));
    });

    group.bench_function("truecolor_to_standard", |b| {
        b.iter(|| black_box(&color_vivid).downgrade(ColorSystem::Standard));
    });

    let color_gray = Color::from_triplet(ColorTriplet::new(128, 128, 128));
    group.bench_function("truecolor_to_256_gray", |b| {
        b.iter(|| black_box(&color_gray).downgrade(ColorSystem::EightBit));
    });

    group.bench_function("parse_and_downgrade", |b| {
        b.iter(|| {
            let color = Color::parse(black_box("#ab52db")).unwrap();
            color.downgrade(ColorSystem::EightBit)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// e) Console rendering
// ---------------------------------------------------------------------------

fn bench_console_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("console_render");

    let console = ConsoleBuilder::new()
        .width(80)
        .force_terminal(true)
        .color_system("truecolor")
        .build();

    // Table with 10 rows
    let mut table_10 = Table::new(&["Name", "Age", "City"]);
    let cities = ["New York", "London", "Tokyo", "Paris", "Berlin"];
    for i in 0..10 {
        table_10.add_row(&[
            &format!("Person {}", i),
            &format!("{}", 20 + i),
            cities[i % 5],
        ]);
    }
    group.bench_function("table_10_rows", |b| {
        b.iter(|| console.render(black_box(&table_10), None));
    });

    // Table with 100 rows
    let mut table_100 = Table::new(&["ID", "Name", "Score", "Status"]);
    for i in 0..100 {
        table_100.add_row(&[
            &format!("{}", i),
            &format!("Entry {}", i),
            &format!("{:.2}", i as f64 * 1.7),
            if i % 2 == 0 { "active" } else { "inactive" },
        ]);
    }
    group.bench_function("table_100_rows", |b| {
        b.iter(|| console.render(black_box(&table_100), None));
    });

    // Panel
    let panel_text = Text::new("Hello, World! This is a panel benchmark.", Style::null());
    let panel = Panel::new(panel_text);
    group.bench_function("panel_simple", |b| {
        b.iter(|| console.render(black_box(&panel), None));
    });

    // Styled panel
    let styled_panel_text = Text::from_markup(
        "[bold red]Warning:[/bold red] Something [italic]important[/italic] happened.",
    )
    .unwrap();
    let styled_panel = Panel::new(styled_panel_text)
        .title(Text::new(
            "Alert",
            Style::parse("bold white on red").unwrap(),
        ))
        .style(Style::parse("red").unwrap());
    group.bench_function("panel_styled", |b| {
        b.iter(|| console.render(black_box(&styled_panel), None));
    });

    // render_lines
    let mut table_lines = Table::new(&["Col A", "Col B"]);
    for i in 0..10 {
        table_lines.add_row(&[&format!("Row {} A", i), &format!("Row {} B", i)]);
    }
    group.bench_function("render_lines_table", |b| {
        b.iter(|| console.render_lines(black_box(&table_lines), None, None, true, true));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// f) Highlighter
// ---------------------------------------------------------------------------

fn bench_highlighter(c: &mut Criterion) {
    let mut group = c.benchmark_group("highlighter");

    let highlighter = ReprHighlighter::new();

    let complex_input = r#"{'name': 'John', 'age': 42, 'active': True, 'scores': [98.5, 87.3, None], 'id': '550e8400-e29b-41d4-a716-446655440000'}"#;
    group.bench_function("repr_complex", |b| {
        b.iter(|| highlighter.apply(black_box(complex_input)));
    });

    let numbers_input = "values = [1, 2.5, -3, 0xff, 1e10, 3.14159, 42, 0b1010, 0o777]";
    group.bench_function("repr_numbers", |b| {
        b.iter(|| highlighter.apply(black_box(numbers_input)));
    });

    group.bench_function("repr_short", |b| {
        b.iter(|| highlighter.apply(black_box("x = 42")));
    });

    let obj_input = r#"User(name='Alice', age=30, email='alice@example.com', active=True)"#;
    group.bench_function("repr_inplace", |b| {
        b.iter_batched(
            || Text::new(obj_input, Style::null()),
            |mut text| highlighter.highlight(black_box(&mut text)),
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// g) Markup parsing
// ---------------------------------------------------------------------------

fn bench_markup_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("markup_parsing");

    group.bench_function("simple", |b| {
        b.iter(|| Text::from_markup(black_box("[bold]Hello[/bold]")).unwrap());
    });

    group.bench_function("nested_deep", |b| {
        b.iter(|| {
            Text::from_markup(black_box(
                "[bold][italic][underline][red]deep[/red][/underline][/italic][/bold]",
            ))
            .unwrap()
        });
    });

    group.bench_function("many_spans", |b| {
        b.iter(|| {
            Text::from_markup(black_box(
                "[red]R[/red][green]G[/green][blue]B[/blue]\
                 [bold]B[/bold][italic]I[/italic][underline]U[/underline]\
                 [red on white]RW[/red on white][bold italic]BI[/bold italic]\
                 [dim]D[/dim][strike]S[/strike]",
            ))
            .unwrap()
        });
    });

    group.bench_function("hex_colors", |b| {
        b.iter(|| {
            Text::from_markup(black_box(
                "[#ff0000]Red[/#ff0000] [#00ff00]Green[/#00ff00] [#0000ff]Blue[/#0000ff] \
                 [rgb(255,128,0)]Orange[/rgb(255,128,0)]",
            ))
            .unwrap()
        });
    });

    let long_markup = (0..50)
        .map(|i| {
            if i % 3 == 0 {
                format!("[bold]word{}[/bold] ", i)
            } else if i % 3 == 1 {
                format!("[italic red]word{}[/italic red] ", i)
            } else {
                format!("word{} ", i)
            }
        })
        .collect::<String>();
    group.bench_function("long_50_words", |b| {
        b.iter(|| Text::from_markup(black_box(&*long_markup)).unwrap());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion group and main
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_style_parsing,
    bench_text_creation,
    bench_text_operations,
    bench_color_downgrade,
    bench_console_render,
    bench_highlighter,
    bench_markup_parsing,
);
criterion_main!(benches);
