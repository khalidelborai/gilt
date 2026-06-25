//! Verifies the gilt **2.3.1** Layout fixes — a `Tree` sidebar inside a split
//! `Layout`, rendered under `Live::with_screen`:
//!
//!   - before 2.3.1 the `Tree` showed only its root (children cropped by the
//!     fixed-size region), and
//!   - the split columns collapsed/staircased in a raw-mode alt-screen TUI
//!     (bare `\n` with no carriage return).
//!
//! Run it:  cargo run --example layout_live
//!
//! It renders a small alt-screen dashboard (header / Tree-sidebar | main / footer)
//! for a couple of seconds, then restores the screen and prints what to look for.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::layout::Layout;
use gilt::live::Live;
use gilt::markdown::Markdown;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;
use gilt::tree::Tree;

fn main() {
    let console = Console::builder()
        .width(82)
        .height(18)
        .force_terminal(true)
        .no_color(false)
        .build();
    let mut live = Live::new(Text::new("", Style::null()))
        .with_console(console)
        .with_screen(true) // alternate-screen path — the one 2.3.1 fixed
        .with_auto_refresh(false);

    let steps = ["scan repo", "edit src", "run tests", "commit"];

    live.start();
    for f in 0..28u32 {
        let i = (f as usize / 7).min(steps.len() - 1);

        // Sidebar: a file Tree WITH CHILDREN (2.3.1 fix #1 — children must survive
        // a fixed-size Layout region).
        let mut tree = Tree::new(Text::new("repo/", Style::parse("bold blue")));
        {
            let src = tree.add(Text::new("src/", Style::parse("bold blue")));
            src.add(Text::new("main.rs", Style::parse("green")));
            src.add(Text::new("lib.rs", Style::parse("green")));
        }
        tree.add(Text::new("Cargo.toml", Style::parse("dim")));
        tree.add(Text::new("README.md", Style::parse("dim")));
        let sidebar = Layout::new(None, Some("sidebar".into()), Some(26), None, None, None)
            .with_renderable(Panel::new(tree).with_title(Text::new("files", Style::parse("dim"))));

        // Main pane.
        let md = format!(
            "## agent\n\nstep **{}** of {}\n\n`{}` …",
            i + 1,
            steps.len(),
            steps[i]
        );
        let main = Layout::new(None, Some("main".into()), None, None, Some(1), None)
            .with_renderable(
                Panel::new(Markdown::new(&md)).with_title(Text::new("work", Style::parse("cyan"))),
            );

        // body = sidebar | main  (split_row — the geometry 2.3.1 fix #2 protects)
        let mut body = Layout::new(None, Some("body".into()), None, None, None, None);
        body.split_row(vec![sidebar, main]);

        let header = Layout::new(None, Some("header".into()), Some(3), None, None, None)
            .with_renderable(Panel::new(Text::new(
                "gilt 2.3.1 — Tree sidebar in a split Layout under Live::with_screen",
                Style::parse("bold magenta"),
            )));
        let footer = Layout::new(None, Some("footer".into()), Some(3), None, None, None)
            .with_renderable(Panel::new(Text::new(
                &format!("frame {f}  ·  children visible + columns aligned = fixed"),
                Style::parse("dim"),
            )));

        let mut root = Layout::new(None, Some("root".into()), None, None, None, None);
        root.split_column(vec![header, body, footer]);

        live.set(root);
        live.refresh();
        thread::sleep(Duration::from_millis(100));
    }
    live.stop();

    println!(
        "✔ 2.3.1 check: the sidebar should have shown ALL tree children \
         (src/ → main.rs, lib.rs; Cargo.toml; README.md) and the two columns \
         should have stayed aligned (no staircase / floating borders)."
    );
}
