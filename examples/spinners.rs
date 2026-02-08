//! All spinners showcase -- displays every available spinner style.
//!
//! Run with: `cargo run --example spinners`
//!
//! Port of Python rich's spinners.py demo. Shows a table of all
//! spinner names along with their frame counts and intervals.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::spinners::SPINNERS;
use gilt::table::Table;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("All Spinners"));

    // Collect spinner names and sort them.
    let mut names: Vec<&&str> = SPINNERS.keys().collect();
    names.sort();

    let mut table = Table::new(&["Name", "Frames", "Interval (ms)", "Preview"]);
    table.title = Some("Available Spinners".to_string());

    for name in &names {
        let data = &SPINNERS[**name];
        let frame_count = data.frames.len().to_string();
        let interval = format!("{:.0}", data.interval);
        // Show first few frames as a preview.
        let preview: String = data
            .frames
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let preview = if data.frames.len() > 8 {
            format!("{} ...", preview)
        } else {
            preview
        };
        table.add_row(&[name, &frame_count, &interval, &preview]);
    }

    console.print(&table);

    console.print(&Rule::with_title("Summary"));
    console.log(&format!("Total spinners: {}", names.len()));
}
