//! Pause/resume handoff — a sticky footer `Live` steps aside for a child
//! display, then comes back without leaving a stale footer in the scrollback.
//!
//! Demonstrates [`Live::pause`] / [`Live::resume`]: the footer is erased in
//! place (no trailing newline pushed into the scrollback), a child `Live`
//! renders a short "subtask" tree on the freed bottom rows, and the footer is
//! redrawn afterwards. Without pause/resume you would toggle `transient` around
//! `stop`/`start` and rebuild the footer by hand.
//!
//! Run with: `cargo run --example live_pause_resume`

use gilt::live::Live;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;
use gilt::tree::Tree;
use std::thread;
use std::time::Duration;

fn footer(status: &str) -> Panel {
    let body = Text::from_markup(&format!(
        "[bold]gilt[/bold]  ·  [cyan]{status}[/cyan]  ·  [dim]ctrl-c to quit[/dim]"
    ))
    .expect("valid markup");
    Panel::new(body).with_border_style(Style::parse("blue"))
}

fn main() {
    // The sticky footer (parent live region).
    let mut footer_live = Live::from_renderable(footer("idle")).with_auto_refresh(false);
    footer_live.start();
    footer_live.update_renderable(footer("running build"), true);
    thread::sleep(Duration::from_millis(900));

    // A subtask needs the bottom rows: pause the footer (erased in place, no
    // stale scrollback line), then let a child Live render there.
    footer_live.pause();

    let mut child = Live::from_renderable(Text::new("spawning subagents…", Style::null()))
        .with_auto_refresh(false);
    child.start();
    for step in 1..=3 {
        let mut tree = Tree::new(Text::from_markup("[bold]spawn_subagents[/bold]").unwrap());
        for i in 1..=step {
            tree.add(Text::from_markup(&format!("[green]✓[/green] agent {i} finished")).unwrap());
        }
        if step < 3 {
            tree.add(
                Text::from_markup(&format!("[yellow]…[/yellow] agent {} working", step + 1))
                    .unwrap(),
            );
        }
        child.update_renderable(tree, true);
        thread::sleep(Duration::from_millis(700));
    }
    // Leave the child's final tree in the scrollback, then resume the footer.
    child.print_above(Text::from_markup("[dim]subagents complete[/dim]").unwrap());
    child.stop();

    // The footer returns exactly where it left off — no duplicate panels.
    footer_live.resume();
    footer_live.update_renderable(footer("build complete"), true);
    thread::sleep(Duration::from_millis(900));

    footer_live.stop();
}
