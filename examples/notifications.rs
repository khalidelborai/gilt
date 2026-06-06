//! Desktop notifications (OSC 9) and taskbar progress (OSC 9;4).
//!
//! These emit terminal escape sequences; the visible effect depends on terminal
//! support — iTerm2 / WezTerm / kitty show notifications, ConEmu / Windows
//! Terminal show taskbar progress. On terminals without support they are
//! harmless no-ops (and they are suppressed on non-terminals / dumb terminals).
//!
//! Run with: `cargo run --example notifications`

use gilt::console::Console;
use gilt::segment::TaskbarState;
use std::thread;
use std::time::Duration;

fn main() {
    let mut console = Console::new();
    console.print_text("[bold]Working…[/] (watch your taskbar)");

    for pct in (0u8..=100).step_by(10) {
        console.set_taskbar_progress(TaskbarState::Normal, pct);
        thread::sleep(Duration::from_millis(200));
    }
    console.set_taskbar_progress(TaskbarState::Remove, 0);

    console.notify("gilt", "Task complete \u{2713}");
    console.print_text("[green]done[/] — sent an OSC 9 desktop notification");
}
