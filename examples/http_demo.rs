//! HTTP download demo with progress tracking.
//!
//! Run with: `cargo run --example http_demo --features http`
//!
//! This demo shows how to use the gilt HTTP module to download files
//! with a curl-style progress bar showing:
//! - Transfer speed (MB/s)
//! - Downloaded size / Total size
//! - ETA (estimated time remaining)
//! - Visual progress bar

use std::env;
use std::path::Path;

use gilt::http::RequestBuilderProgress;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let url = if args.len() >= 2 {
        args[1].clone()
    } else {
        // Default: download a small test file from httpbin.org
        "https://httpbin.org/bytes/102400".to_string()
    };

    println!("Gilt HTTP Demo");
    println!("==============\n");

    // Example 1: Simple GET request with progress
    println!("Example 1: Simple GET request");
    println!("URL: {}", url);
    println!();

    let response = reqwest::Client::new()
        .get(&url)
        .with_progress("Downloading data")
        .send()
        .await?;

    println!("\nStatus: {}", response.status());

    if let Some(content_length) = response.content_length() {
        println!("Content-Length: {} bytes", content_length);
    } else {
        println!("Content-Length: unknown (chunked transfer)");
    }

    // Read the body
    let bytes = response.bytes().await?;
    println!("Downloaded: {} bytes\n", bytes.len());

    // Example 2: Using convenience function
    if args.len() >= 3 {
        let download_url = &args[1];
        let output_path = &args[2];

        println!("Example 2: Download to file");
        println!("URL: {}", download_url);
        println!("Output: {}", output_path);
        println!();

        let bytes_written =
            gilt::http::download_with_progress(download_url, Path::new(output_path), "Saving file")
                .await?;

        println!("\nSaved {} bytes to {}\n", bytes_written, output_path);
    }

    // Example 3: JSON API with progress
    println!("Example 3: JSON API request");
    println!("Fetching JSON data from httpbin.org...\n");

    let json_response = reqwest::Client::new()
        .get("https://httpbin.org/json")
        .with_progress("Fetching JSON")
        .send()
        .await?;

    let json_text = json_response.text().await?;
    println!("\nJSON Response (truncated):");
    if json_text.len() > 200 {
        println!("{}...", &json_text[..200]);
    } else {
        println!("{}", json_text);
    }

    println!("\nAll examples completed!");
    Ok(())
}
