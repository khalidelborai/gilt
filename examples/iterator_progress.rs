//! Demonstrates gilt's `.progress()` iterator adapter — wrap any iterator
//! with a live progress bar in a single method call.
//!
//! Run with: `cargo run --example iterator_progress`

use gilt::progress::ProgressIteratorExt;
use std::thread;
use std::time::Duration;

fn main() {
    // ── Range iterator with .progress() ──────────────────────────────────
    println!("=== Range iterator: (0..50).progress() ===\n");

    let sum: i32 = (0..50)
        .progress("Processing items")
        .inspect(|_| {
            thread::sleep(Duration::from_millis(30));
        })
        .sum();

    println!("\nSum of 0..50 = {sum}\n");

    // ── Vec iterator with .progress() ────────────────────────────────────
    println!("=== Vec iterator: vec.iter().progress() ===\n");

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

    println!(
        "\nCollected {} items: {:?}\n",
        collected.len(),
        &collected[..5]
    );

    // ── Explicit total with .progress_with_total() ───────────────────────
    println!("=== Explicit total: .progress_with_total() ===\n");

    let items: Vec<u64> = (0..30)
        .progress_with_total("Crunching numbers", 30.0)
        .map(|n| {
            thread::sleep(Duration::from_millis(25));
            n * n
        })
        .collect();

    println!("\nSquares: {:?}...", &items[..5]);
    println!("All {} items yielded correctly.\n", items.len());

    println!("Done! The iterator adapter yields every item as normal —");
    println!("the progress bar is just a visual side-effect.");
}
