//! Demonstrates the ProgressIteratorExt for adding progress bars to any iterator.
//!
//! Run with: `cargo run --example iterator_progress_demo`

use std::thread;
use std::time::Duration;

use gilt::progress::ProgressIteratorExt;

fn main() {
    // ── 1. Basic range iterator with .progress() ────────────────────────
    //
    // The total is inferred from the Range's size_hint (100 items).
    println!("=== Range iterator ===\n");

    let sum: i32 = (0..100)
        .progress("Counting")
        .inspect(|_| {
            thread::sleep(Duration::from_millis(20));
        })
        .sum();

    println!("\nSum of 0..100 = {}\n", sum);

    // ── 2. Vec iterator with .progress() ────────────────────────────────
    //
    // Vec::iter() provides an exact size_hint, so the bar knows the total.
    println!("=== Vec iterator ===\n");

    let fruits = ["apple", "banana", "cherry", "date", "elderberry", "fig"];

    let uppercased: Vec<String> = fruits
        .iter()
        .progress("Processing fruits")
        .map(|fruit| {
            thread::sleep(Duration::from_millis(300));
            fruit.to_uppercase()
        })
        .collect();

    println!("\nResults: {}\n", uppercased.join(", "));

    // ── 3. Explicit total with .progress_with_total() ───────────────────
    //
    // When the iterator's size_hint is not available or you want to override
    // it, use progress_with_total() to set an explicit total.
    println!("=== Explicit total ===\n");

    let items = vec![10, 20, 30, 40, 50];

    let doubled: Vec<i32> = items
        .into_iter()
        .progress_with_total("Doubling values", 5.0)
        .map(|x| {
            thread::sleep(Duration::from_millis(250));
            x * 2
        })
        .collect();

    println!("\nDoubled: {:?}\n", doubled);

    println!("All progress demos complete.");
}
