//! Status spinner demo -- shows a spinner with status messages while "doing work".
//!
//! Run with: `cargo run --example status`
//!
//! Port of Python rich's status.py demo. Demonstrates the Status widget
//! which combines a Spinner with a Live display for animated feedback.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::status::Status;

fn main() {
    let console = Console::builder()
        .force_terminal(true)
        .no_color(false)
        .build();

    let tasks = [
        "Downloading data",
        "Processing files",
        "Training model",
        "Evaluating results",
        "Generating report",
    ];

    let mut status = Status::new("Getting ready...")
        .with_console(console);
    status.start();

    for (i, task) in tasks.iter().enumerate() {
        // Update the status message for the current task.
        status
            .update()
            .status(&format!("{} ({}/{})...", task, i + 1, tasks.len()))
            .apply()
            .unwrap();

        // Simulate work.
        thread::sleep(Duration::from_secs(1));

        eprintln!("  Done: {}", task);
    }

    status.stop();

    eprintln!();
    eprintln!("All tasks complete!");
}
