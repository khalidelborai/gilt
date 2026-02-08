//! Progress bar example -- demonstrates long-running task tracking.
//!
//! Run with: `cargo run --example progress`
//!
//! Shows three concurrent tasks with different speeds, similar to
//! Python rich's download progress demo.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::progress::Progress;

fn main() {
    let console = Console::builder().width(80).force_terminal(true).build();

    let mut progress = Progress::new(Progress::default_columns())
        .with_console(console)
        .with_auto_refresh(false);

    // Simulate three concurrent downloads at different speeds.
    let task1 = progress.add_task("Downloading dataset.tar.gz", Some(1000.0));
    let task2 = progress.add_task("Downloading model-weights.bin", Some(500.0));
    let task3 = progress.add_task("Downloading config.json", Some(200.0));

    progress.start();

    let mut done1 = false;
    let mut done2 = false;
    let mut done3 = false;

    loop {
        // Task 1: fast (20 units per tick)
        if !done1 {
            progress.advance(task1, 20.0);
            if let Some(t) = progress.get_task(task1) {
                if t.finished() {
                    done1 = true;
                }
            }
        }

        // Task 2: medium (8 units per tick)
        if !done2 {
            progress.advance(task2, 8.0);
            if let Some(t) = progress.get_task(task2) {
                if t.finished() {
                    done2 = true;
                }
            }
        }

        // Task 3: slow (3 units per tick)
        if !done3 {
            progress.advance(task3, 3.0);
            if let Some(t) = progress.get_task(task3) {
                if t.finished() {
                    done3 = true;
                }
            }
        }

        progress.refresh();

        if done1 && done2 && done3 {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    progress.stop();

    println!("\nAll downloads complete!");
}
