//! Truecolor gradient progress bars (`BarColumn::with_gradient`).
//!
//! The completed portion of the bar is filled with a per-cell 24-bit gradient
//! interpolated between two colors.
//!
//! Run with: `cargo run --example gradient_progress`

use gilt::color::Color;
use gilt::progress::{BarColumn, Progress, ProgressColumn, TaskProgressColumn, TextColumn};
use std::thread;
use std::time::Duration;

fn main() {
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(
            BarColumn::new()
                .with_gradient(Color::from_rgb(255, 80, 0), Color::from_rgb(0, 200, 255)),
        ),
        Box::new(TaskProgressColumn::default()),
    ];

    let mut progress = Progress::new(columns);
    let task = progress.add_task("Rendering", Some(100.0), true);

    progress.start();
    for _ in 0..100 {
        progress.advance(task, 1.0);
        thread::sleep(Duration::from_millis(30));
    }
    progress.stop();
}
