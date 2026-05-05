//! Status spinner demo. Run: `cargo run --example status`

use gilt::status::Status;
use std::{thread, time::Duration};

fn main() {
    let mut status = Status::run("Getting ready...");
    for n in 1..=10 {
        status.set(&format!("task {n}"));
        thread::sleep(Duration::from_secs(1));
        eprintln!("  task {n} complete");
    }
}
