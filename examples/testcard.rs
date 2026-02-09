//! Test card example -- showcases gilt's features in a single grid table.
//!
//! Inspired by Python rich's `__main__.py` test card, this example builds one
//! large grid table where each row demonstrates a feature category, then renders
//! it with cold/warm timing.
//!
//! Run with: `cargo run --example testcard --all-features`

use std::time::Instant;

use gilt::box_chars::SIMPLE;
use gilt::color::Color;
use gilt::color_triplet::ColorTriplet;
use gilt::console::{Console, ConsoleOptions, Renderable};
use gilt::emoji_replace::emoji_replace;
use gilt::gradient::Gradient;
use gilt::panel::Panel;
use gilt::progress_bar::ProgressBar;
use gilt::segment::Segment;
use gilt::style::Style;
use gilt::table::{ColumnOptions, Table};
use gilt::text::{JustifyMethod, Text};
use gilt::tree::Tree;

// ---------------------------------------------------------------------------
// ColorBox -- a renderable that paints an HSL gradient using half-block chars
// ---------------------------------------------------------------------------

/// HLS-to-RGB conversion matching Python's `colorsys.hls_to_rgb`.
/// H, L, S are in [0.0, 1.0]; returns (R, G, B) in [0.0, 1.0].
fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let m2 = if l <= 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let m1 = 2.0 * l - m2;
    (
        v(m1, m2, h + 1.0 / 3.0),
        v(m1, m2, h),
        v(m1, m2, h - 1.0 / 3.0),
    )
}

fn v(m1: f64, m2: f64, mut hue: f64) -> f64 {
    if hue < 0.0 {
        hue += 1.0;
    }
    if hue > 1.0 {
        hue -= 1.0;
    }
    if hue < 1.0 / 6.0 {
        m1 + (m2 - m1) * hue * 6.0
    } else if hue < 0.5 {
        m2
    } else if hue < 2.0 / 3.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
    } else {
        m1
    }
}

/// A renderable that paints 5 rows of HSL-interpolated half-block characters.
struct ColorBox;

impl Renderable for ColorBox {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();
        let width = options.max_width;
        for y in 0..5u32 {
            for x in 0..width {
                let h = x as f64 / width as f64;
                let l1 = 0.1 + (y as f64 / 5.0) * 0.7;
                let l2 = l1 + 0.7 / 10.0;
                let (r1, g1, b1) = hls_to_rgb(h, l1, 1.0);
                let (r2, g2, b2) = hls_to_rgb(h, l2, 1.0);
                let bgcolor = Color::from_triplet(ColorTriplet::new(
                    (r1 * 255.0) as u8,
                    (g1 * 255.0) as u8,
                    (b1 * 255.0) as u8,
                ));
                let fgcolor = Color::from_triplet(ColorTriplet::new(
                    (r2 * 255.0) as u8,
                    (g2 * 255.0) as u8,
                    (b2 * 255.0) as u8,
                ));
                let style = Style::from_color(Some(fgcolor), Some(bgcolor));
                segments.push(Segment::styled("\u{2584}", style)); // ▄
            }
            segments.push(Segment::line());
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Helper: render a Renderable to Text via capture
// ---------------------------------------------------------------------------

/// Render any `Renderable` widget into a `Text` object by capturing its ANSI
/// output and decoding it back.
fn render_to_text(console: &mut Console, widget: &dyn Renderable) -> Text {
    console.begin_capture();
    console.print(widget);
    let captured = console.end_capture();
    // Strip the trailing newline that print() adds
    let trimmed = captured.trim_end_matches('\n');
    Text::from_ansi(trimmed)
}

// ---------------------------------------------------------------------------
// Helper: side-by-side comparison table
// ---------------------------------------------------------------------------

/// Create a grid table with two columns of equal ratio for side-by-side display.
fn comparison(console: &mut Console, left: &dyn Renderable, right: &dyn Renderable) -> Text {
    let left_text = render_to_text(console, left);
    let right_text = render_to_text(console, right);

    let mut table = Table::grid(&[]);
    table.set_expand(true);
    table.padding = (0, 1, 0, 0);
    table.add_column(
        "",
        "",
        ColumnOptions {
            ratio: Some(1),
            ..Default::default()
        },
    );
    table.add_column(
        "",
        "",
        ColumnOptions {
            ratio: Some(1),
            ..Default::default()
        },
    );
    table.add_row_text(&[left_text, right_text]);

    render_to_text(console, &table)
}

// ---------------------------------------------------------------------------
// make_test_card
// ---------------------------------------------------------------------------

fn make_test_card(console: &mut Console) -> Table {
    let mut table = Table::grid(&[]);
    table.title = Some("gilt features".to_string());
    table.padding = (1, 1, 0, 1);
    table.pad_edge = true;
    table.set_expand(true);
    table.add_column(
        "Feature",
        "",
        ColumnOptions {
            no_wrap: true,
            justify: Some(JustifyMethod::Center),
            style: Some("bold red".to_string()),
            ..Default::default()
        },
    );
    table.add_column("Demonstration", "", Default::default());

    // ── Row 1: Colors ────────────────────────────────────────────────────
    {
        let color_text = render_to_text(
            console,
            &Text::from_markup(
                "\u{2713} [bold green]4-bit color[/]\n\
                 \u{2713} [bold blue]8-bit color[/]\n\
                 \u{2713} [bold magenta]Truecolor (16.7 million)[/]\n\
                 \u{2713} [bold yellow]Dumb terminals[/]\n\
                 \u{2713} [bold cyan]Automatic color conversion",
            )
            .unwrap(),
        );
        let color_box_text = render_to_text(console, &ColorBox);

        let mut color_table = Table::grid(&[]);
        color_table.padding = (0, 0, 0, 0);
        color_table.add_column("", "", Default::default());
        color_table.add_column("", "", Default::default());
        color_table.add_row_text(&[color_text, color_box_text]);

        let demo = render_to_text(console, &color_table);
        table.add_row_text(&[Text::from_markup("[bold red]Colors[/]").unwrap(), demo]);
    }

    // ── Row 2: Styles ────────────────────────────────────────────────────
    table.add_row(&[
        "Styles",
        "All ANSI styles: [bold]bold[/], [dim]dim[/], [italic]italic[/italic], \
         [underline]underline[/], [strike]strikethrough[/], [reverse]reverse[/], \
         and even [blink]blink[/].",
    ]);

    // ── Row 3: Text (4 justify modes) ────────────────────────────────────
    {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                     Quisque in metus sed sapien ultricies pretium a at justo. \
                     Maecenas luctus velit et auctor maximus.";

        let mut t_left = Text::new(lorem, Style::parse("green").unwrap());
        t_left.justify = Some(JustifyMethod::Left);
        let mut t_center = Text::new(lorem, Style::parse("yellow").unwrap());
        t_center.justify = Some(JustifyMethod::Center);
        let mut t_right = Text::new(lorem, Style::parse("blue").unwrap());
        t_right.justify = Some(JustifyMethod::Right);
        let mut t_full = Text::new(lorem, Style::parse("red").unwrap());
        t_full.justify = Some(JustifyMethod::Full);

        let mut lorem_table = Table::grid(&[]);
        lorem_table.padding = (0, 1, 0, 0);
        lorem_table.collapse_padding = true;
        lorem_table.set_expand(true);
        for _ in 0..4 {
            lorem_table.add_column(
                "",
                "",
                ColumnOptions {
                    ratio: Some(1),
                    ..Default::default()
                },
            );
        }
        lorem_table.add_row_text(&[t_left, t_center, t_right, t_full]);

        let header = Text::from_markup(
            "Word wrap text. Justify [green]left[/], [yellow]center[/], \
             [blue]right[/], or [red]full[/].\n",
        )
        .unwrap();
        let header_rendered = render_to_text(console, &header);
        let lorem_rendered = render_to_text(console, &lorem_table);

        // Combine header + lorem table into one Text
        let mut combined = header_rendered;
        combined.append_str("\n", None);
        combined.append_text(&lorem_rendered);

        table.add_row_text(&[Text::from_markup("[bold red]Text[/]").unwrap(), combined]);
    }

    // ── Row 4: CJK Support ──────────────────────────────────────────────
    {
        let cjk_text = emoji_replace(
            ":flag_for_china:  \u{8be5}\u{5e93}\u{652f}\u{6301}\u{4e2d}\u{6587}\u{ff0c}\
             \u{65e5}\u{6587}\u{548c}\u{97e9}\u{6587}\u{6587}\u{672c}\u{ff01}\n\
             :flag_for_japan:  \u{30e9}\u{30a4}\u{30d6}\u{30e9}\u{30ea}\u{306f}\u{4e2d}\u{56fd}\u{8a9e}\u{3001}\
             \u{65e5}\u{672c}\u{8a9e}\u{3001}\u{97d3}\u{56fd}\u{8a9e}\u{306e}\u{30c6}\u{30ad}\u{30b9}\u{30c8}\u{3092}\
             \u{30b5}\u{30dd}\u{30fc}\u{30c8}\u{3057}\u{3066}\u{3044}\u{307e}\u{3059}\n\
             :flag_for_south_korea:  \u{c774} \u{b77c}\u{c774}\u{be0c}\u{b7ec}\u{b9ac}\u{b294} \
             \u{c911}\u{ad6d}\u{c5b4}, \u{c77c}\u{bca8}\u{c5b4} \u{bc0f} \u{d55c}\u{ad6d}\u{c5b4} \
             \u{d14d}\u{c2a4}\u{d2b8}\u{b97c} \u{c9c0}\u{c6d0}\u{d569}\u{b2c8}\u{b2e4}",
            None,
        );
        table.add_row_text(&[
            Text::new("CJK\nSupport", Style::null()),
            Text::new(&cjk_text, Style::null()),
        ]);
    }

    // ── Row 5: Markup ───────────────────────────────────────────────────
    {
        let markup_text = emoji_replace(
            "[bold magenta]gilt[/] supports a simple [i]bbcode[/i]-like [b]markup[/b] \
             for [yellow]color[/], [underline]style[/], and emoji! \
             :thumbs_up: :apple: :ant: :bear: :baguette_bread: :bus:",
            None,
        );
        table.add_row(&["Markup", &markup_text]);
    }

    // ── Row 6: Tables ───────────────────────────────────────────────────
    {
        let mut movie_table = Table::new(&[]);
        movie_table.show_edge = false;
        movie_table.show_header = true;
        movie_table.set_expand(false);
        movie_table.row_styles = vec!["".to_string(), "dim".to_string()];
        movie_table.box_chars = Some(&SIMPLE);

        movie_table.add_column(
            "[green]Date",
            "",
            ColumnOptions {
                style: Some("green".to_string()),
                no_wrap: true,
                ..Default::default()
            },
        );
        movie_table.add_column(
            "[blue]Title",
            "",
            ColumnOptions {
                style: Some("blue".to_string()),
                ..Default::default()
            },
        );
        movie_table.add_column(
            "[cyan]Production Budget",
            "",
            ColumnOptions {
                style: Some("cyan".to_string()),
                justify: Some(JustifyMethod::Right),
                no_wrap: true,
                ..Default::default()
            },
        );
        movie_table.add_column(
            "[magenta]Box Office",
            "",
            ColumnOptions {
                style: Some("magenta".to_string()),
                justify: Some(JustifyMethod::Right),
                no_wrap: true,
                ..Default::default()
            },
        );

        movie_table.add_row(&[
            "Dec 20, 2019",
            "Star Wars: The Rise of Skywalker",
            "$275,000,000",
            "$375,126,118",
        ]);
        movie_table.add_row(&[
            "May 25, 2018",
            "[b]Solo[/]: A Star Wars Story",
            "$275,000,000",
            "$393,151,347",
        ]);
        movie_table.add_row(&[
            "Dec 15, 2017",
            "Star Wars Ep. VIII: The Last Jedi",
            "$262,000,000",
            "[bold]$1,332,539,889[/bold]",
        ]);
        movie_table.add_row(&[
            "May 19, 1999",
            "Star Wars Ep. [b]I[/b]: [i]The Phantom Menace",
            "$115,000,000",
            "$1,027,044,677",
        ]);

        let demo = render_to_text(console, &movie_table);
        table.add_row_text(&[Text::from_markup("[bold red]Tables[/]").unwrap(), demo]);
    }

    // ── Row 7: Syntax & Pretty ──────────────────────────────────────────
    {
        #[cfg(feature = "syntax")]
        {
            use gilt::syntax::Syntax;

            let code = "\
fn iter_last<T>(values: impl Iterator<Item = T>) -> impl Iterator<Item = (bool, T)> {
    let mut iter = values.peekable();
    std::iter::from_fn(move || {
        iter.next().map(|val| (iter.peek().is_none(), val))
    })
}";
            let syntax = Syntax::new(code, "rs")
                .with_line_numbers(true)
                .with_indent_guides(true);

            use gilt::pretty::Pretty;

            let data = serde_json::json!({
                "foo": [
                    3.1427,
                    [
                        "Paul Atreides",
                        "Vladimir Harkonnen",
                        "Thufir Hawat"
                    ]
                ],
                "atomic": [false, true, null]
            });
            let pretty = Pretty::from_json(&data);

            let demo = comparison(console, &syntax, &pretty);
            table.add_row_text(&[
                Text::from_markup("[bold red]Syntax\nhighlighting\n&\npretty\nprinting[/]")
                    .unwrap(),
                demo,
            ]);
        }

        #[cfg(not(feature = "syntax"))]
        {
            table.add_row(&[
                "Syntax\nhighlighting\n&\npretty\nprinting",
                "[dim](enable 'syntax' feature to see this demo)[/]",
            ]);
        }
    }

    // ── Row 8: Markdown ─────────────────────────────────────────────────
    {
        #[cfg(feature = "markdown")]
        {
            use gilt::markdown::Markdown;

            let markdown_source = "\
# Markdown

Supports much of the *markdown* __syntax__!

- Headers
- Basic formatting: **bold**, *italic*, `code`
- Block quotes
- Lists, and more...
";
            let left = Text::new(markdown_source, Style::parse("cyan").unwrap());
            let md = Markdown::new(markdown_source);

            let demo = comparison(console, &left, &md);
            table.add_row_text(&[Text::from_markup("[bold red]Markdown[/]").unwrap(), demo]);
        }

        #[cfg(not(feature = "markdown"))]
        {
            table.add_row(&[
                "Markdown",
                "[dim](enable 'markdown' feature to see this demo)[/]",
            ]);
        }
    }

    // ── Row 9: Tree ─────────────────────────────────────────────────────
    {
        let bold_blue = Style::parse("bold blue").unwrap();
        let default = Style::null();

        let mut tree = Tree::new(Text::new("gilt/", bold_blue.clone()))
            .with_guide_style(Style::parse("dim green").unwrap());

        let src = tree.add(Text::new("src/", bold_blue.clone()));
        src.add(Text::new("console.rs", default.clone()));
        src.add(Text::new("table.rs", default.clone()));
        src.add(Text::new("text.rs", default.clone()));
        {
            let widgets = src.add(Text::new("widgets/", bold_blue.clone()));
            widgets.add(Text::new("panel.rs", default.clone()));
            widgets.add(Text::new("tree.rs", default.clone()));
        }

        let examples = tree.add(Text::new("examples/", bold_blue));
        examples.add(Text::new("testcard.rs", default.clone()));
        examples.add(Text::new("table_movie.rs", default));

        tree.add(Text::new("Cargo.toml", Style::parse("dim").unwrap()));

        let demo = render_to_text(console, &tree);
        table.add_row_text(&[Text::from_markup("[bold red]Tree[/]").unwrap(), demo]);
    }

    // ── Row 10: Progress Bars ───────────────────────────────────────────
    {
        let levels: &[(f64, &str)] = &[
            (0.0, "[dim]  0%[/] "),
            (30.0, "[yellow] 30%[/] "),
            (70.0, "[blue] 70%[/] "),
            (100.0, "[bold green]100%[/] "),
        ];

        let mut bar_text = Text::empty();
        for (completed, label) in levels {
            let label_t = Text::from_markup(label).unwrap();
            bar_text.append_text(&label_t);

            let bar = ProgressBar::new()
                .with_total(Some(100.0))
                .with_completed(*completed)
                .with_width(Some(30));
            let bar_rendered = render_to_text(console, &bar);
            bar_text.append_text(&bar_rendered);
            bar_text.append_str("\n", None);
        }

        table.add_row_text(&[
            Text::from_markup("[bold red]Progress[/]").unwrap(),
            bar_text,
        ]);
    }

    // ── Row 11: Emoji ───────────────────────────────────────────────────
    {
        let emoji_text = emoji_replace(
            "Emoji codes: :rocket: :sparkles: :thumbs_up: :heart: :star: :fire: \
             :white_check_mark: :cross_mark: :warning: :hourglass:",
            None,
        );
        table.add_row(&["Emoji", &emoji_text]);
    }

    // ── Row 12: Gradient ────────────────────────────────────────────────
    {
        let rainbow = Gradient::rainbow(
            "The quick brown fox jumps over the lazy dog — gilt supports beautiful rainbow gradients!",
        );
        let demo = render_to_text(console, &rainbow);
        table.add_row_text(&[Text::from_markup("[bold red]Gradient[/]").unwrap(), demo]);
    }

    // ── Row 13: +more! ──────────────────────────────────────────────────
    table.add_row(&[
        "+more!",
        "Progress bars, columns, layouts, live display, status spinners, \
         logging, tracebacks, accessibility, and more...",
    ]);

    table
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    // Build a capture console for timing (force truecolor, same width as real terminal)
    let (term_width, _) = Console::detect_terminal_size();

    let mut capture_console = Console::builder()
        .width(term_width)
        .force_terminal(true)
        .color_system("truecolor")
        .build();

    // Build the test card (this populates the card using capture_console)
    let test_card = make_test_card(&mut capture_console);

    // Cold render (first render, populating caches)
    let cold_start = Instant::now();
    capture_console.begin_capture();
    capture_console.print(&test_card);
    let _ = capture_console.end_capture();
    let cold_ms = cold_start.elapsed().as_secs_f64() * 1000.0;

    // Warm render
    let warm_start = Instant::now();
    capture_console.begin_capture();
    capture_console.print(&test_card);
    let _ = capture_console.end_capture();
    let warm_ms = warm_start.elapsed().as_secs_f64() * 1000.0;

    // Now print for real
    let mut console = Console::builder()
        .force_terminal(true)
        .color_system("truecolor")
        .build();

    console.print(&test_card);

    // Timing
    console.print_text(&format!(
        "\n[dim]Rendered in [not dim]{cold_ms:.1}ms[/] (cold) / [not dim]{warm_ms:.1}ms[/] (warm)[/]"
    ));

    // Farewell panel
    let farewell = Panel::new(
        Text::from_markup(
            "[bold magenta]gilt[/] \u{2014} Rich terminal formatting for Rust\n\n\
             [cyan]https://github.com/khalidelborai/gilt[/]",
        )
        .unwrap(),
    )
    .with_title(Text::styled(
        " gilt v0.3.0 ",
        Style::parse("bold green").unwrap(),
    ))
    .with_border_style(Style::parse("green").unwrap());
    console.print(&farewell);
}
