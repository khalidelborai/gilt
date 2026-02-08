//! Simulated file download with progress bars.
//!
//! Run with: `cargo run --example downloader`
//!
//! Port of Python rich's downloader.py. Simulates downloading multiple files
//! with progress bars showing transfer speed and file size columns.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::progress::{
    BarColumn, DownloadColumn, Progress, TextColumn, TransferSpeedColumn, TimeRemainingColumn,
};

/// Simulated file with a name and size in bytes.
struct Download {
    name: &'static str,
    size: f64,
    speed: f64, // bytes per tick
}

fn main() {
    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Build a download-style progress display with file size columns.
    let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new()),
        Box::new(DownloadColumn),
        Box::new(TransferSpeedColumn),
        Box::new(TimeRemainingColumn::new()),
    ];

    let mut progress = Progress::new(columns)
        .with_console(console)
        .with_auto_refresh(false);

    // Simulate several file downloads at different speeds.
    let downloads = [
        Download {
            name: "linux-6.8.tar.xz",
            size: 143_654_912.0,
            speed: 4_500_000.0,
        },
        Download {
            name: "rust-1.77-src.tar.gz",
            size: 89_200_640.0,
            speed: 2_800_000.0,
        },
        Download {
            name: "node-v20.11.1.tar.gz",
            size: 52_428_800.0,
            speed: 6_100_000.0,
        },
        Download {
            name: "python-3.12.2.tgz",
            size: 25_600_000.0,
            speed: 1_200_000.0,
        },
    ];

    // Add all download tasks.
    let task_ids: Vec<_> = downloads
        .iter()
        .map(|d| progress.add_task(d.name, Some(d.size)))
        .collect();

    progress.start();

    loop {
        let mut all_done = true;

        for (i, dl) in downloads.iter().enumerate() {
            if let Some(task) = progress.get_task(task_ids[i]) {
                if !task.finished() {
                    all_done = false;
                }
            }
            // Advance even if finished -- advance is clamped to total.
            progress.advance(task_ids[i], dl.speed);
        }

        progress.refresh();

        if all_done {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    progress.stop();

    println!("\nAll downloads complete!");
}
