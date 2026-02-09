//! Progress column types demonstration
//!
//! This example demonstrates all the available progress column types
//! including the new SpinnerColumn, TimeRemainingColumn, TimeElapsedColumn,
//! FileSizeColumn, DownloadColumn, and TransferSpeedColumn.
//!
//! Run: cargo run --example progress_columns_demo

use gilt::prelude::*;
use gilt::progress::{
    BarColumn, DownloadColumn, FileSizeColumn, Progress, SpinnerColumn,
    TextColumn, TimeElapsedColumn, TimeRemainingColumn, TransferSpeedColumn,
};
use std::thread;
use std::time::Duration;

fn main() {
    let mut console = Console::new();

    console.rule(Some("Progress Columns Demo"));
    console.print_text("Demonstrating all progress column types\n");

    // ========================================================================
    // 1. Spinner Column Demo
    // ========================================================================
    console.rule(Some("1. Spinner Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(SpinnerColumn::new("dots")),
            Box::new(TextColumn::new("{task.description}")),
        ];
        let mut progress = Progress::new(columns);
        let task = progress.add_task("Loading with spinner", None);
        progress.start();

        for _ in 0..30 {
            progress.advance(task, 1.0);
            thread::sleep(Duration::from_millis(50));
        }
        progress.stop();
    }

    // ========================================================================
    // 2. Time Elapsed Column
    // ========================================================================
    console.rule(Some("2. Time Elapsed Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(TimeElapsedColumn),
        ];
        let mut progress = Progress::new(columns);
        let task = progress.add_task("Processing with elapsed time", Some(50.0));
        progress.start();

        for _ in 0..50 {
            progress.advance(task, 1.0);
            thread::sleep(Duration::from_millis(30));
        }
        progress.stop();
    }

    // ========================================================================
    // 3. Time Remaining Column
    // ========================================================================
    console.rule(Some("3. Time Remaining Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(TimeRemainingColumn::default()),
        ];
        let mut progress = Progress::new(columns);
        let task = progress.add_task("Processing with ETA", Some(50.0));
        progress.start();

        for _ in 0..50 {
            progress.advance(task, 1.0);
            thread::sleep(Duration::from_millis(30));
        }
        progress.stop();
    }

    // ========================================================================
    // 4. File Size Column
    // ========================================================================
    console.rule(Some("4. File Size Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(SpinnerColumn::new("dots")),
            Box::new(TextColumn::new("{task.description}")),
            Box::new(FileSizeColumn::default()),
        ];
        let mut progress = Progress::new(columns);
        let task = progress.add_task("Uploading file", None);
        progress.start();

        for _ in 0..40 {
            progress.advance(task, 25600.0); // 25KB chunks
            thread::sleep(Duration::from_millis(50));
        }
        progress.stop();
    }

    // ========================================================================
    // 5. Download Column (completed/total)
    // ========================================================================
    console.rule(Some("5. Download Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(DownloadColumn::new()),
        ];
        let mut progress = Progress::new(columns);
        let total = 10.0 * 1024.0 * 1024.0; // 10 MB
        let task = progress.add_task("Downloading", Some(total));
        progress.start();

        for _ in 0..50 {
            progress.advance(task, total / 50.0);
            thread::sleep(Duration::from_millis(30));
        }
        progress.stop();
    }

    // ========================================================================
    // 6. Transfer Speed Column
    // ========================================================================
    console.rule(Some("6. Transfer Speed Column"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(TransferSpeedColumn::new()),
        ];
        let mut progress = Progress::new(columns);
        let total = 50.0 * 1024.0 * 1024.0; // 50 MB
        let task = progress.add_task("Transferring", Some(total));
        progress.start();

        for _ in 0..50 {
            progress.advance(task, total / 50.0);
            thread::sleep(Duration::from_millis(30));
        }
        progress.stop();
    }

    // ========================================================================
    // 7. Combined: Full Download Progress
    // ========================================================================
    console.rule(Some("7. Full Download Progress"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(SpinnerColumn::new("moon")),
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(DownloadColumn::new()),
            Box::new(TransferSpeedColumn::new()),
            Box::new(TimeRemainingColumn::default()),
        ];
        let mut progress = Progress::new(columns);
        let total = 100.0 * 1024.0 * 1024.0; // 100 MB
        let task = progress.add_task("Mega download", Some(total));
        progress.start();

        for _ in 0..60 {
            progress.advance(task, total / 60.0);
            thread::sleep(Duration::from_millis(25));
        }
        progress.stop();
    }

    // ========================================================================
    // 8. Multi-Task with Different Columns
    // ========================================================================
    console.rule(Some("8. Multi-Task Display"));

    {
        let columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
            Box::new(TextColumn::new("{task.description}")),
            Box::new(BarColumn::new()),
            Box::new(TimeElapsedColumn),
            Box::new(TimeRemainingColumn::default()),
        ];
        let mut progress = Progress::new(columns);

        let task1 = progress.add_task("Task A", Some(100.0));
        let task2 = progress.add_task("Task B", Some(100.0));
        let task3 = progress.add_task("Task C", Some(100.0));

        progress.start();

        for i in 0..100 {
            progress.advance(task1, 1.0);
            if i % 2 == 0 {
                progress.advance(task2, 1.0);
            }
            if i % 3 == 0 {
                progress.advance(task3, 1.0);
            }
            thread::sleep(Duration::from_millis(20));
        }
        progress.stop();
    }

    console.line(1);
    console.print_text("[green]✓[/green] Progress columns demo complete!");
}
