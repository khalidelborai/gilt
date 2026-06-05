//! Live status `Panel` — a framed box whose contents update in place.
//!
//! Demonstrates v1.5.2 `Live` holding a `Panel`, re-rendered through the live
//! console each frame so the border stretches to the real terminal width and
//! the body reflows inside it.
//!
//! Run with: `cargo run --example live_status_panel`

use gilt::live::Live;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;
use std::thread;
use std::time::Duration;

fn main() {
    let files = [
        "index.html",
        "app.js",
        "styles.css",
        "logo.svg",
        "data.json",
    ];

    let mut live = Live::from_renderable(Text::new("", Style::null())).with_auto_refresh(false);
    live.start();

    for (i, file) in files.iter().enumerate() {
        let done = i + 1;
        let pct = done * 100 / files.len();
        let filled = pct / 5; // 20-cell bar
        let bar = format!(
            "{}{}",
            "\u{2588}".repeat(filled),      // █
            "\u{2591}".repeat(20 - filled), // ░
        );

        let body = Text::from_markup(&format!(
            "Installing [cyan bold]{file}[/cyan bold]\n\n[green]{bar}[/green] {pct}%\n\n[dim]{done} of {} files complete[/dim]",
            files.len(),
        ))
        .expect("valid markup");

        let panel = Panel::new(body)
            .with_title("Installer")
            .with_border_style(Style::parse("cyan"));

        live.update_renderable(panel, true);
        thread::sleep(Duration::from_millis(550));
    }

    thread::sleep(Duration::from_millis(800));
    live.stop();
}
