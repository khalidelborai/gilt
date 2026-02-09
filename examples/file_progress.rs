//! Read a file with progress tracking and display a summary.
//!
//! Run with: `cargo run --example file_progress -- path/to/file.txt`
//!
//! If no file is provided, a temporary demo file is created and read instead.
//!
//! Adapted from Python rich's file_progress.py. Reads a file line by line
//! while showing a progress bar, then prints the line count and file size.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::process;
use std::thread;
use std::time::Duration;

use gilt::filesize;
use gilt::progress::{
    BarColumn, FileSizeColumn, Progress, ProgressColumn, TextColumn, TimeRemainingColumn,
    TotalFileSizeColumn, TransferSpeedColumn,
};

/// Create a temporary demo file with sample content and return its path.
fn create_demo_file() -> io::Result<String> {
    let path = std::env::temp_dir().join("gilt_file_progress_demo.txt");
    let mut f = File::create(&path)?;
    for i in 1..=200 {
        writeln!(f, "Line {i}: The quick brown fox jumps over the lazy dog.")?;
    }
    f.flush()?;
    Ok(path.to_string_lossy().into_owned())
}

fn read_with_progress(file_path: &str) -> io::Result<usize> {
    let metadata = fs::metadata(file_path)?;
    let total_bytes = metadata.len() as f64;

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Build file-read progress columns: description, bar, completed/total size, speed, ETA.
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new()),
        Box::new(FileSizeColumn),
        Box::new(TotalFileSizeColumn),
        Box::new(TransferSpeedColumn),
        Box::new(TimeRemainingColumn::new()),
    ];

    let mut progress = Progress::new(columns).with_auto_refresh(false);

    let description = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    let task_id = progress.add_task(description, Some(total_bytes));
    progress.start();

    let mut line_count = 0usize;
    for line in reader.lines() {
        let line = line?;
        // Advance by the byte length of the line plus the newline character.
        let byte_len = line.len() as f64 + 1.0;
        progress.advance(task_id, byte_len);
        line_count += 1;

        progress.refresh();

        // Small sleep so the progress display is visible for small files.
        thread::sleep(Duration::from_millis(1));
    }

    progress.stop();

    let size_str = filesize::decimal(total_bytes as u64, 1, " ");
    println!(
        "\nRead {} lines ({}) from {}",
        line_count, size_str, file_path
    );

    Ok(line_count)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file_path: String = if args.len() >= 2 {
        args[1].clone()
    } else {
        println!("No file specified -- creating a temporary demo file.\n");
        match create_demo_file() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error creating demo file: {e}");
                process::exit(1);
            }
        }
    };

    match read_with_progress(&file_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
