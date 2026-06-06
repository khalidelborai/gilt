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
//! ```

mod cmd;

use clap::{Parser, Subcommand};
use cmd::StyleOpts;
use std::io;

// ---------------------------------------------------------------------------
// CLI definition (clap derive)
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "gilt",
    about = "Rich terminal output for scripts and CI",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print text with rich markup (e.g. `[bold red]hi[/]`)
    Print {
        /// Markup string to render
        markup: String,
    },

    /// Print text with explicit style flags
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
    Table,

    /// Draw a horizontal rule, optionally with a title
    Rule {
        /// Optional title to display in the center of the rule
        title: Option<String>,
    },

    /// Render text inside a bordered panel
    Panel {
        /// Content text for the panel
        text: String,

        /// Optional panel title
        #[arg(long)]
        title: Option<String>,
    },

    /// Read Markdown from stdin and render it
    Markdown,

    /// Read JSON from stdin and pretty-print it
    Json,
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
    };

    if let Err(e) = result {
        eprintln!("gilt: error: {e}");
        std::process::exit(1);
    }
}
