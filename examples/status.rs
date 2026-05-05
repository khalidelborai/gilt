//! Status spinner demo. Run: `cargo run --example status`

use gilt::{console::Console, status::Status};
use std::{thread, time::Duration};

fn main() {
    let tasks = [
        "Downloading",
        "Processing",
        "Training",
        "Evaluating",
        "Reporting",
    ];
    let mut status = Status::new("Getting ready...").with_console(Console::default());
    status.start();
    for (i, t) in tasks.iter().enumerate() {
        status
            .update()
            .status(&format!("{t} ({}/{})", i + 1, tasks.len()))
            .apply()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        eprintln!("  Done: {t}");
    }
    status.stop();
    eprintln!("\nAll tasks complete!");
}
