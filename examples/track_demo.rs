//! Iterator progress tracking demonstration
//!
//! Run: cargo run --example track_demo

use gilt::prelude::*;
use gilt::progress::{track, ProgressIteratorExt, SpinnerColumn, TimeRemainingColumn};
use std::thread;
use std::time::Duration;

fn main() {
    let mut console = Console::new();

    console.rule(Some("Track Demo"));

    // ── 1. Basic track() with range iterator ─────────────────────────────────
    println!("\n[1] Basic track() with range (0..50)");
    println!("    Using track(iter, description, total)\n");

    let sum: i32 = track(0..50, "Processing range", Some(50.0))
        .inspect(|_| {
            thread::sleep(Duration::from_millis(30));
        })
        .sum();

    println!("\n    Sum of 0..50 = {}\n", sum);

    // ── 2. track() with vector iteration ─────────────────────────────────────
    println!("[2] track() with Vec iteration");
    println!("    Tracking processing of string items\n");

    let items = vec![
        "apple",
        "banana",
        "cherry",
        "date",
        "elderberry",
        "fig",
        "grape",
    ];

    let uppercased: Vec<String> =
        track(items.iter(), "Processing fruits", Some(items.len() as f64))
            .map(|fruit| {
                thread::sleep(Duration::from_millis(200));
                fruit.to_uppercase()
            })
            .collect();

    println!("\n    Results: {:?}\n", uppercased);

    // ── 3. track() without total (indeterminate progress) ────────────────────
    println!("[3] track() without total (indeterminate/spinner mode)");
    println!("    Shows spinner when total is unknown\n");

    let items: Vec<i32> = track(0..30, "Unknown total work", None)
        .map(|n| {
            thread::sleep(Duration::from_millis(50));
            n * 2
        })
        .collect();

    println!("\n    Collected {} items\n", items.len());

    // ── 4. ProgressIteratorExt: .progress() on range ─────────────────────────
    println!("[4] ProgressIteratorExt: .progress() method on range");
    println!("    Total inferred from size_hint()\n");

    let count = (0..40)
        .progress("Counting with .progress()")
        .inspect(|_| {
            thread::sleep(Duration::from_millis(25));
        })
        .count();

    println!("\n    Counted {} items\n", count);

    // ── 5. ProgressIteratorExt: .progress() on Vec ───────────────────────────
    println!("[5] ProgressIteratorExt: .progress() on Vec iterator");
    println!("    Automatically detects length from Vec\n");

    let data = vec![
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra",
        "tango",
    ];

    let collected: Vec<&str> = data
        .iter()
        .copied()
        .progress("Loading data")
        .inspect(|_| {
            thread::sleep(Duration::from_millis(40));
        })
        .collect();

    println!("\n    Collected {} items\n", collected.len());

    // ── 6. .progress_with_total() for explicit total ─────────────────────────
    println!("[6] .progress_with_total() with explicit total");
    println!("    Override or specify total explicitly\n");

    // Simulating processing where we know the total even if iterator doesn't report it
    let results: Vec<i32> = (0..25)
        .progress_with_total("Processing with explicit total", 25.0)
        .map(|n| {
            thread::sleep(Duration::from_millis(30));
            n * n
        })
        .collect();

    println!("\n    Processed {} items\n", results.len());

    // ── 7. Custom column configuration ───────────────────────────────────────
    println!("[7] Custom column configuration with track()");
    println!("    Using custom columns: spinner + bar + time remaining\n");

    use gilt::progress::{BarColumn, Progress, TextColumn};

    let custom_columns: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
        Box::new(SpinnerColumn::new("dots")),
        Box::new(
            TextColumn::new("{task.description}")
                .with_style(Style::parse("bold cyan").unwrap_or_else(|_| Style::null())),
        ),
        Box::new(BarColumn::default()),
        Box::new(TimeRemainingColumn::new()),
    ];

    let mut progress = Progress::new(custom_columns);
    let task_id = progress.add_task("Custom columns task", Some(30.0));
    progress.start();

    for _ in 0..30 {
        thread::sleep(Duration::from_millis(50));
        progress.advance(task_id, 1.0);
        progress.refresh();
    }
    progress.stop();

    println!("\n    Custom column demo complete!\n");

    // ── 8. Multiple tasks with track() ───────────────────────────────────────
    println!("[8] Sequential tasks - processing stages");
    println!("    Each stage tracked independently\n");

    // Stage 1: Download
    let downloaded: Vec<u8> = track(0..20, "Stage 1: Downloading", Some(20.0))
        .map(|_| {
            thread::sleep(Duration::from_millis(40));
            1u8
        })
        .collect();

    println!("    Downloaded {} bytes\n", downloaded.len());

    // Stage 2: Process
    let processed: Vec<u8> = track(downloaded.into_iter(), "Stage 2: Processing", Some(20.0))
        .map(|b| {
            thread::sleep(Duration::from_millis(30));
            b.wrapping_add(1)
        })
        .collect();

    println!("    Processed {} bytes\n", processed.len());

    // Stage 3: Upload
    let _uploaded: Vec<u8> = track(processed.into_iter(), "Stage 3: Uploading", Some(20.0))
        .map(|b| {
            thread::sleep(Duration::from_millis(35));
            b
        })
        .collect();

    println!("    Upload complete!\n");

    // ── 9. Spinner column variations ─────────────────────────────────────────
    println!("[9] Different spinner styles");
    println!("    Using different spinner animations\n");

    // dots spinner
    let custom_columns_dots: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
        Box::new(SpinnerColumn::new("dots")),
        Box::new(TextColumn::new("dots spinner")),
    ];
    let mut progress = Progress::new(custom_columns_dots);
    let task_id = progress.add_task("", Some(10.0));
    progress.start();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(80));
        progress.advance(task_id, 1.0);
        progress.refresh();
    }
    progress.stop();

    // line spinner
    let custom_columns_line: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
        Box::new(SpinnerColumn::new("line")),
        Box::new(TextColumn::new("line spinner")),
    ];
    let mut progress = Progress::new(custom_columns_line);
    let task_id = progress.add_task("", Some(10.0));
    progress.start();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(80));
        progress.advance(task_id, 1.0);
        progress.refresh();
    }
    progress.stop();

    // moon spinner
    let custom_columns_moon: Vec<Box<dyn gilt::progress::ProgressColumn>> = vec![
        Box::new(SpinnerColumn::new("moon")),
        Box::new(TextColumn::new("moon spinner")),
    ];
    let mut progress = Progress::new(custom_columns_moon);
    let task_id = progress.add_task("", Some(10.0));
    progress.start();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(80));
        progress.advance(task_id, 1.0);
        progress.refresh();
    }
    progress.stop();

    println!("\n    All spinner demos complete!\n");

    // ── 10. Summary ──────────────────────────────────────────────────────────
    console.rule(Some("Summary"));
    println!("");
    println!("The track() function and ProgressIteratorExt trait provide");
    println!("flexible progress tracking for iterators:");
    println!("");
    println!("  • track(iter, desc, total) - wraps any iterator with progress");
    println!("  • iter.progress(desc) - extension method on iterators");
    println!("  • iter.progress_with_total(desc, total) - explicit total");
    println!("  • Custom column configurations for different visual styles");
    println!("  • Automatic progress bar or spinner based on total availability");
    println!("");
    println!("All methods automatically start/stop the live display and");
    println!("advance the progress bar as items are yielded.");
    println!("");
}
