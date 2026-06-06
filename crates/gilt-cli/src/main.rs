//! gilt — CLI binary for rich terminal output.
//!
//! Use gilt from shell scripts, Makefiles, and CI without writing Rust:
//!
//! ```text
//! gilt print '[bold]hi[/]'
//! gilt style --fg red --bold 'warning'
//! gilt table < data.csv
//! gilt rule 'Section'
//! gilt panel 'text' --title Title
//! gilt markdown < README.md
//! gilt json < data.json
//! gilt tree < outline.txt
//! gilt syntax --lang rust < main.rs
//! gilt completions bash
//! ```

mod cmd;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use cmd::StyleOpts;
use std::io;

// ---------------------------------------------------------------------------
// CLI definition (clap derive)
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "gilt",
    about = "Rich terminal output for scripts and CI",
    long_about = "gilt renders styled text, tables, trees, syntax-highlighted code, \
                  and markdown to your terminal — from shell scripts, Makefiles, and \
                  CI pipelines, without writing Rust.\n\n\
                  All output is ANSI escape sequences compatible with any modern \
                  terminal. Use `gilt completions <shell>` to enable tab-completion.",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print text with rich markup (e.g. `[bold red]hi[/]`)
    #[command(about = "Print text with rich markup tags")]
    Print {
        /// Markup string to render
        markup: String,
    },

    /// Print text with explicit style flags
    #[command(about = "Print text with explicit style flags (--fg, --bg, --bold, …)")]
    Style {
        /// Foreground color (e.g. red, #ff0000, 196)
        #[arg(long)]
        fg: Option<String>,

        /// Background color (e.g. blue, #0000ff)
        #[arg(long)]
        bg: Option<String>,

        /// Bold text
        #[arg(long)]
        bold: bool,

        /// Italic text
        #[arg(long)]
        italic: bool,

        /// Underline text
        #[arg(long)]
        underline: bool,

        /// Dim text
        #[arg(long)]
        dim: bool,

        /// Text to style
        text: String,
    },

    /// Read CSV from stdin and render as a table
    #[command(about = "Read CSV from stdin and render as a Unicode box-drawing table")]
    Table,

    /// Draw a horizontal rule, optionally with a title
    #[command(about = "Draw a horizontal rule (optionally with a centered title)")]
    Rule {
        /// Optional title to display in the center of the rule
        title: Option<String>,
    },

    /// Render text inside a bordered panel
    #[command(about = "Render text inside a bordered panel")]
    Panel {
        /// Content text for the panel
        text: String,

        /// Optional panel title
        #[arg(long)]
        title: Option<String>,
    },

    /// Read Markdown from stdin and render it
    #[command(about = "Read Markdown from stdin and render it to the terminal")]
    Markdown,

    /// Read JSON from stdin and pretty-print it
    #[command(about = "Read JSON from stdin and pretty-print it with syntax highlighting")]
    Json,

    /// Read an indented outline from stdin and render as a tree
    ///
    /// Each line is a node. Indent by multiples of 2 spaces to set depth.
    /// The first line is the root.
    ///
    /// Example input:
    ///   Project/
    ///     src/
    ///       main.rs
    ///     Cargo.toml
    #[command(
        about = "Read an indented outline from stdin and render as a tree",
        long_about = "Read an indented text outline from stdin and render it as a Tree.\n\n\
                      Each line is a node. Indent by multiples of 2 spaces to set depth.\n\
                      The first non-empty line becomes the root.\n\n\
                      Example:\n  echo -e 'Project/\\n  src/\\n    main.rs\\n  Cargo.toml' | gilt tree"
    )]
    Tree,

    /// Read code from stdin and render with syntax highlighting
    #[command(about = "Read code from stdin and render with syntax highlighting")]
    Syntax {
        /// Language name or file extension (e.g. rust, py, js, toml)
        #[arg(long, short)]
        lang: String,

        /// Color theme name (default: base16-ocean.dark)
        #[arg(long, default_value = "base16-ocean.dark")]
        theme: String,

        /// Show line numbers
        #[arg(long)]
        line_numbers: bool,
    },

    /// Emit a shell completion script
    #[command(about = "Emit a shell completion script for bash, zsh, or fish")]
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let result: io::Result<()> = match cli.command {
        Commands::Print { markup } => cmd::cmd_print(&markup, &mut out),

        Commands::Style {
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
            text,
        } => {
            let opts = StyleOpts {
                fg,
                bg,
                bold,
                italic,
                underline,
                dim,
            };
            cmd::cmd_style(&opts, &text, &mut out)
        }

        Commands::Table => cmd::cmd_table(io::stdin().lock(), &mut out),

        Commands::Rule { title } => cmd::cmd_rule(title.as_deref(), &mut out),

        Commands::Panel { text, title } => cmd::cmd_panel(&text, title.as_deref(), &mut out),

        Commands::Markdown => cmd::cmd_markdown(io::stdin().lock(), &mut out),

        Commands::Json => cmd::cmd_json(io::stdin().lock(), &mut out),

        Commands::Tree => cmd::cmd_tree(io::stdin().lock(), &mut out),

        Commands::Syntax {
            lang,
            theme,
            line_numbers,
        } => cmd::cmd_syntax(io::stdin().lock(), &lang, &theme, line_numbers, &mut out),

        Commands::Completions { shell } => {
            let mut app = Cli::command();
            clap_complete::generate(shell, &mut app, "gilt", &mut out);
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("gilt: error: {e}");
        std::process::exit(1);
    }
}
