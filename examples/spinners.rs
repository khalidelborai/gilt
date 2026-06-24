//! All spinners live demo. Run: `cargo run --example spinners`
//!
//! Mirrors rich/examples/spinners.py: every available spinner laid out in
//! columns inside a live-refreshing panel.

use gilt::columns::Columns;
use gilt::live::Live;
use gilt::panel::Panel;
use gilt::spinners::SPINNERS;
use gilt::status::spinner::Spinner;
use std::{thread, time::Duration};

fn main() {
    let mut names: Vec<&&str> = SPINNERS.keys().collect();
    names.sort();
    let spinners: Vec<Spinner> = names.iter().filter_map(|n| Spinner::new(n).ok()).collect();
    let panel = Panel::from_renderable(
        Columns::from_renderables(spinners)
            .with_column_first(true)
            .with_expand(true),
    )
    .with_title("Spinners");

    let live = Live::from_renderable(panel).with_refresh_per_second(20.0);
    let mut live = live;
    live.start();
    thread::sleep(Duration::from_secs(5));
    live.stop();
}
