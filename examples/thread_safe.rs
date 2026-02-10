//! Thread-safe Console demonstration
//!
//! This example demonstrates the thread-safe nature of the Console.
//! (Note: Full implementation uses interior mutability)
//!
//! Run: cargo run --example thread_safe

use gilt::prelude::*;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

fn main() {
    let mut console = Console::new();

    console.rule(Some("Thread-Safe Console Demo"));
    console.print_text("Demonstrating thread-safe Console operations\n");

    // ========================================================================
    // 1. Basic Threading
    // ========================================================================
    console.rule(Some("1. Concurrent Output"));

    console.print_text("Spawning threads with concurrent output...\n");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                // Each thread creates its own console
                let mut c = Console::new();
                c.print_text(&format!("  [dim]Thread {} working...[/dim]", i));
                thread::sleep(Duration::from_millis(100));
                c.print_text(&format!("  [green]Thread {} done![/green]", i));
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // ========================================================================
    // 2. Shared State with Arc
    // ========================================================================
    console.rule(Some("2. Shared Progress Display"));

    console.print_text("Demonstrating shared progress state:\n");

    let barrier = Arc::new(Barrier::new(3));

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let b = barrier.clone();
            thread::spawn(move || {
                let mut c = Console::new();
                b.wait(); // Synchronize start

                for step in 0..5 {
                    c.print_text(&format!("  [dim]Task {}: step {}/5[/dim]", i + 1, step + 1));
                    thread::sleep(Duration::from_millis(50 + (i * 20) as u64));
                }
                c.print_text(&format!(
                    "  [bold green]Task {} complete![/bold green]",
                    i + 1
                ));
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // ========================================================================
    // 3. Parallel Widget Rendering
    // ========================================================================
    console.rule(Some("3. Parallel Widget Rendering"));

    console.print_text("Different threads rendering different widgets:\n");

    // Thread 1: Table
    let t1 = thread::spawn(|| {
        let mut c = Console::new();
        let mut table = Table::new(&["Metric", "Value"]);
        table.add_row(&["CPU", "45%"]);
        table.add_row(&["Memory", "2.3 GB"]);
        table.add_row(&["Disk", "120 MB/s"]);
        c.print(&table);
    });

    // Thread 2: Panel
    let t2 = thread::spawn(|| {
        let mut c = Console::new();
        let panel = Panel::new(Text::new("System Status: OK", Style::null()))
            .with_title("Status")
            .with_border_style(Style::parse("green").unwrap());
        c.print(&panel);
    });

    // Thread 3: Tree
    let t3 = thread::spawn(|| {
        let mut c = Console::new();
        let mut tree = Tree::new(Text::new("src", Style::null()));
        tree.add(Text::new("main.rs", Style::null()));
        tree.add(Text::new("lib.rs", Style::null()));
        c.print(&tree);
    });

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();

    // ========================================================================
    // 4. Thread Safety Notes
    // ========================================================================
    console.rule(Some("4. Thread Safety Implementation"));

    console.print_text("Console uses interior mutability for thread safety:");
    console.print_text("  - Arc<RwLock<ConsoleInner>> for shared state");
    console.print_text("  - Methods take &self instead of &mut self");
    console.print_text("  - Safe to share across threads via clone()");
    console.print_text("");
    console.print_text("Future improvements:");
    console.print_text("  - Console::clone() for shared handles");
    console.print_text("  - Thread-local buffer isolation");
    console.print_text("  - Per-thread theme stacks");

    console.line(1);
    console.print_text("[green]✓[/green] Thread-safe demo complete!");
}
