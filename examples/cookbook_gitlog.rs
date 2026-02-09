//! Cookbook: Git Log Viewer
//!
//! A rich git log display with commit graph, branch colors, author info,
//! and diff stats. Shows how Panel, Table, Tree, Rule, Text, Style, and
//! Gradient work together to produce a polished `git log` alternative.
//!
//! Run with: `cargo run --example cookbook_gitlog`

use gilt::box_chars::{DOUBLE, HEAVY, ROUNDED};
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Header Panel ──────────────────────────────────────────────────────
    let title_gradient = Gradient::two_color(
        "  Git Log: myproject  ",
        Color::from_rgb(255, 180, 50),
        Color::from_rgb(255, 100, 200),
    )
    .with_style(Style::parse("bold").unwrap());

    let header = Panel::new(Text::new(
        "Branch:        main\n\
         Commits:       847\n\
         Contributors:  12\n\
         Last tag:      v2.3.1 (3 days ago)",
        Style::null(),
    ))
    .with_title(Text::styled(
        " myproject ",
        Style::parse("bold bright_white on blue").unwrap(),
    ))
    .with_subtitle(Text::styled(
        "HEAD -> main",
        Style::parse("bold green").unwrap(),
    ))
    .with_border_style(Style::parse("bright_yellow").unwrap())
    .with_box_chars(&DOUBLE);

    console.print(&title_gradient);
    console.print(&header);

    // ── Commit History Table ──────────────────────────────────────────────
    console.print(
        &Rule::with_title("Commit History").with_style(Style::parse("bright_yellow").unwrap()),
    );

    let mut table = Table::new(&["Hash", "Graph", "Message", "Author", "Date", "Files"]);
    table.header_style = "bold bright_white on grey23".to_string();
    table.border_style = "bright_yellow".to_string();
    table.box_chars = Some(&ROUNDED);

    // (hash, graph, prefix, message, author, date, additions, deletions)
    #[allow(clippy::type_complexity)]
    let commits: &[(&str, &str, &str, &str, &str, &str, u32, u32)] = &[
        (
            "a3f7b21",
            "*  ",
            "feat",
            "add OAuth2 login flow",
            "Alice Chen",
            "2 hours ago",
            148,
            12,
        ),
        (
            "e9c4d08",
            "*  ",
            "fix",
            "resolve race condition in session store",
            "Bob Park",
            "5 hours ago",
            23,
            31,
        ),
        (
            "1b8e3f5",
            "|\\ ",
            "feat",
            "implement rate limiter middleware",
            "Carol Diaz",
            "1 day ago",
            210,
            5,
        ),
        (
            "7d2a9c1",
            "| *",
            "docs",
            "update API reference for v2.3",
            "Dave Kim",
            "1 day ago",
            85,
            42,
        ),
        (
            "f4e6b83",
            "|/ ",
            "refactor",
            "extract config into dedicated module",
            "Alice Chen",
            "2 days ago",
            67,
            89,
        ),
        (
            "3c1d7a9",
            "*  ",
            "fix",
            "correct timezone offset in audit log",
            "Eve Novak",
            "2 days ago",
            8,
            4,
        ),
        (
            "b5f2e04",
            "*  ",
            "chore",
            "bump dependencies to latest patch",
            "Frank Liu",
            "3 days ago",
            15,
            15,
        ),
        (
            "8a4c6d2",
            "*  ",
            "feat",
            "add WebSocket heartbeat monitor",
            "Carol Diaz",
            "3 days ago",
            193,
            0,
        ),
        (
            "d1e9f37",
            "*  ",
            "refactor",
            "simplify error propagation chain",
            "Bob Park",
            "4 days ago",
            34,
            71,
        ),
        (
            "6f0b8a5",
            "*  ",
            "fix",
            "handle empty payload in webhook handler",
            "Alice Chen",
            "5 days ago",
            11,
            3,
        ),
    ];

    for &(hash, graph, prefix, msg, author, date, add, del) in commits {
        // Hash: dim yellow
        let hash_text = Text::styled(hash, Style::parse("dim yellow").unwrap());

        // Graph: colored branch indicator
        let graph_style = if graph.contains('\\') || graph.contains('/') || graph.contains('|') {
            Style::parse("bold bright_magenta").unwrap()
        } else {
            Style::parse("bold bright_green").unwrap()
        };
        let graph_text = Text::styled(graph, graph_style);

        // Message: prefix colored by convention, rest normal
        let prefix_style = match prefix {
            "feat" => "bold green",
            "fix" => "bold red",
            "refactor" => "bold cyan",
            "docs" => "bold blue",
            "chore" => "dim",
            _ => "",
        };
        let full_msg = format!("[{prefix_style}]{prefix}:[/{prefix_style}] {msg}");
        let msg_text = Text::from_markup(&full_msg)
            .unwrap_or_else(|_| Text::styled(&format!("{prefix}: {msg}"), Style::null()));

        // Author: bold
        let author_text = Text::styled(author, Style::parse("bold").unwrap());

        // Date: dim
        let date_text = Text::styled(date, Style::parse("dim").unwrap());

        // Files: +N in green, -M in red
        let files_str = format!("[green]+{add}[/green] [red]-{del}[/red]");
        let files_text = Text::from_markup(&files_str)
            .unwrap_or_else(|_| Text::styled(&format!("+{add} -{del}"), Style::null()));

        table.add_row_text(&[
            hash_text,
            graph_text,
            msg_text,
            author_text,
            date_text,
            files_text,
        ]);
    }

    console.print(&table);

    // ── Branch Tree ───────────────────────────────────────────────────────
    console.print(&Rule::with_title("Branches").with_style(Style::parse("bright_yellow").unwrap()));

    let bold_yellow = Style::parse("bold bright_yellow").unwrap();
    let bold_green = Style::parse("bold green").unwrap();
    let bold_cyan = Style::parse("bold cyan").unwrap();
    let bold_magenta = Style::parse("bold magenta").unwrap();
    let dim = Style::parse("dim").unwrap();
    let green = Style::parse("green").unwrap();

    let mut tree = Tree::new(Text::styled("origin", bold_yellow.clone()))
        .with_guide_style(Style::parse("bright_yellow").unwrap());

    // main branch
    {
        let main = tree.add(Text::styled("main  \u{2190} HEAD", bold_green.clone()));
        main.add(Text::styled(
            "a3f7b21  feat: add OAuth2 login flow",
            dim.clone(),
        ));
    }

    // feature/auth branch
    {
        let feature = tree.add(Text::styled("feature/auth", bold_cyan.clone()));
        feature.add(Text::styled(
            "1b8e3f5  implement rate limiter middleware",
            dim.clone(),
        ));
        feature.add(Text::styled(
            "\u{2191} 2 ahead, 0 behind main",
            green.clone(),
        ));
    }

    // bugfix/login branch
    {
        let bugfix = tree.add(Text::styled("bugfix/login", bold_magenta.clone()));
        bugfix.add(Text::styled(
            "e9c4d08  resolve race condition in session store",
            dim.clone(),
        ));
        bugfix.add(Text::styled(
            "\u{2191} 1 ahead, 3 behind main",
            Style::parse("yellow").unwrap(),
        ));
    }

    // release/v2.0 branch
    {
        let release = tree.add(Text::styled(
            "release/v2.0",
            Style::parse("bold bright_red").unwrap(),
        ));
        release.add(Text::styled("tagged v2.0.0  (2 weeks ago)", dim.clone()));
        release.add(Text::styled("\u{2714} merged into main", green.clone()));
    }

    console.print(&tree);

    // ── Summary Rule ──────────────────────────────────────────────────────
    console.print(
        &Rule::new()
            .with_characters("\u{2550}")
            .with_style(Style::parse("bright_yellow").unwrap()),
    );

    let summary = Panel::fit(Text::styled(
        "Showing 10 of 847 commits  \u{00b7}  4 branches  \u{00b7}  12 contributors",
        Style::parse("bold").unwrap(),
    ))
    .with_border_style(Style::parse("dim").unwrap())
    .with_box_chars(&HEAVY);
    console.print(&summary);
}
