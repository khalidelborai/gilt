//! Copy a file with a progress bar showing transfer speed and ETA.
//!
//! Run with: `cargo run --example cp_progress -- source.txt dest.txt`
//!
//! Adapted from Python rich's cp_progress.py. Uses manual chunked I/O with
//! gilt's Progress display to show real-time copy progress.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process;
use std::thread;
use std::time::Duration;

use gilt::progress::{
    BarColumn, DownloadColumn, Progress, ProgressColumn, TextColumn, TimeRemainingColumn,
    TransferSpeedColumn,
};

const CHUNK_SIZE: usize = 8 * 1024; // 8 KB

fn copy_with_progress(src_path: &str, dst_path: &str) -> io::Result<()> {
    // Get file size for progress tracking.
    let metadata = fs::metadata(src_path)?;
    let total_bytes = metadata.len() as f64;

    // Open source and destination files.
    let src_file = File::open(src_path)?;
    let dst_file = File::create(dst_path)?;
    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    // Build download-style progress columns.
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new()),
        Box::new(DownloadColumn::new()),
        Box::new(TransferSpeedColumn::new()),
        Box::new(TimeRemainingColumn::new()),
    ];

    let mut progress = Progress::new(columns).with_auto_refresh(false);

    // Derive a short description from the filename.
    let description = std::path::Path::new(src_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(src_path);

    let task_id = progress.add_task(description, Some(total_bytes));
    progress.start();

    let mut buf = [0u8; CHUNK_SIZE];
    loop {
        let bytes_read = reader.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buf[..bytes_read])?;
        progress.advance(task_id, bytes_read as f64);
        progress.refresh();

        // Small sleep so the progress bar is visible even for fast copies.
        thread::sleep(Duration::from_millis(1));
    }

    writer.flush()?;
    progress.stop();

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <source> <destination>", args[0]);
        eprintln!();
        eprintln!("Copy a file with a progress bar showing transfer speed and ETA.");
        process::exit(1);
    }

    let src = &args[1];
    let dst = &args[2];

    match copy_with_progress(src, dst) {
        Ok(()) => {
            println!("\nCopied {} -> {}", src, dst);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
