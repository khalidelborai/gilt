//! Demonstrates a directory listing using gilt's Columns widget.
//!
//! Lists files and directories with color coding: directories appear in bold
//! blue, regular files in the default color. Accepts an optional path argument.
//!
//! Usage: cargo run --example listdir [PATH]

use gilt::columns::Columns;
use gilt::console::Console;
use gilt::rule::Rule;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Determine which directory to list
    let target = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let dir = Path::new(&target);

    let title = format!(
        "Directory: {}",
        dir.canonicalize()
            .unwrap_or_else(|_| dir.to_path_buf())
            .display()
    );
    console.print(&Rule::with_title(&title));

    // Read directory entries and sort them
    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(err) => {
            console.log(&format!("[bold red]Error:[/bold red] {}", err));
            return;
        }
    };
    entries.sort_by_key(|e| e.file_name());

    // Build styled entries for Columns
    let mut cols = Columns::new();

    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden files (starting with '.')
        if name_str.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        if is_dir {
            // Directories: bold blue with trailing slash, using markup
            let styled = format!("[bold blue]{}/[/bold blue]", name_str);
            cols.add_renderable(&styled);
        } else {
            // Files: highlight extension with color
            let styled = match Path::new(&*name_str).extension().and_then(|e| e.to_str()) {
                Some("rs") => format!("[green]{}[/green]", name_str),
                Some("toml") | Some("yaml") | Some("yml") | Some("json") => {
                    format!("[yellow]{}[/yellow]", name_str)
                }
                Some("md") | Some("txt") | Some("rst") => {
                    format!("[cyan]{}[/cyan]", name_str)
                }
                Some("lock") => format!("[dim]{}[/dim]", name_str),
                _ => name_str.to_string(),
            };
            cols.add_renderable(&styled);
        }
    }

    console.print(&cols);

    // Print summary
    let total = entries
        .iter()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .count();
    let dir_count = entries
        .iter()
        .filter(|e| {
            !e.file_name().to_string_lossy().starts_with('.')
                && e.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
        })
        .count();
    let file_count = total - dir_count;

    console.log(&format!(
        "[bold]{}[/bold] items: [bold blue]{}[/bold blue] directories, {} files",
        total, dir_count, file_count
    ));
}
