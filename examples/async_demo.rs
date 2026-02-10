//! Async demo -- demonstrates async progress tracking and live displays.
//!
//! Run with: `cargo run --example async_demo --features async`

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use futures::StreamExt;
    use gilt::r#async::{LiveAsync, ProgressChannel, ProgressStreamExt};
    use gilt::style::Style;
    use gilt::text::Text;
    use std::time::Duration;

    println!("=== Async Demo ===\n");

    // Demo 1: Progress stream
    println!("1. Progress Stream Demo");
    {
        let stream = futures::stream::iter(0..50);
        let mut progress_stream = stream.track_progress("Processing items", Some(50.0));

        while let Some(i) = progress_stream.next().await {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(20)).await;
            if i % 10 == 0 {
                println!("  Reached item {}", i);
            }
        }
    }
    println!();

    // Demo 2: Progress Channel
    println!("2. Progress Channel Demo");
    {
        let (tx, progress) = ProgressChannel::with_total("Downloading", 100.0);

        // Run the progress display first (it will wait for updates)
        let progress_handle = tokio::spawn(async move {
            progress.run().await;
        });

        // Give the progress display a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Spawn a worker task
        let worker = tokio::spawn(async move {
            for i in 0..=100 {
                tx.update(i as f64).await;
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            tx.finish().await;
        });

        // Wait for completion
        let _ = tokio::join!(worker, progress_handle);
    }
    println!();

    // Demo 3: LiveAsync display
    println!("3. Live Async Display Demo");
    {
        let mut live = LiveAsync::new(Text::styled(
            "Initializing...",
            Style::parse("yellow").unwrap(),
        ))
        .with_refresh_interval(Duration::from_millis(100));

        live.start().await;

        for i in 1..=5 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            live.update(Text::styled(
                &format!("Step {}/5...", i),
                Style::parse("cyan").unwrap(),
            ))
            .await;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
        live.update(Text::styled("Complete!", Style::parse("green").unwrap()))
            .await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        live.stop().await;
    }
    println!();

    // Demo 4: Multiple concurrent tasks with ProgressChannel
    println!("4. Multiple Concurrent Tasks Demo");
    {
        let (tx1, progress1) = ProgressChannel::with_total("Task A", 50.0);
        let (tx2, progress2) = ProgressChannel::with_total("Task B", 50.0);

        // Start progress displays first
        let progress_handle1 = tokio::spawn(async move {
            progress1.run().await;
        });

        let progress_handle2 = tokio::spawn(async move {
            progress2.run().await;
        });

        // Give displays a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        let worker1 = tokio::spawn(async move {
            for i in 0..=50 {
                tx1.update(i as f64).await;
                tokio::time::sleep(Duration::from_millis(30)).await;
            }
            tx1.finish().await;
        });

        let worker2 = tokio::spawn(async move {
            for i in 0..=50 {
                tx2.update(i as f64).await;
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            tx2.finish().await;
        });

        let _ = tokio::join!(worker1, worker2, progress_handle1, progress_handle2);
    }

    println!("\n=== Demo Complete ===");
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature enabled.");
    eprintln!("Run with: cargo run --example async_demo --features async");
    std::process::exit(1);
}
