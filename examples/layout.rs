//! Layout demo -- shows split layouts with header, body, sidebar, and footer.
//!
//! Run with: `cargo run --example layout`
//!
//! Port of Python rich's layout.py demo. Demonstrates the Layout widget
//! which divides a fixed-height area into rows and columns.

use gilt::console::Console;
use gilt::layout::Layout;
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .height(24)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Layout Demo"));

    // Build a layout tree:
    //
    //   +---------------------------+
    //   |         HEADER            |  (size: 3)
    //   +--------+------------------+
    //   |        |                  |
    //   | SIDE   |     MAIN         |  (flexible)
    //   | BAR    |                  |
    //   |        |                  |
    //   +--------+------------------+
    //   |         FOOTER            |  (size: 3)
    //   +---------------------------+

    // Create named layouts with content.
    let header = Layout::new(
        Some("HEADER -- gilt layout demo".to_string()),
        Some("header".to_string()),
        Some(3),
        None,
        None,
        None,
    );

    let sidebar = Layout::new(
        Some("SIDEBAR\n  - Nav\n  - Links\n  - Help".to_string()),
        Some("sidebar".to_string()),
        Some(20),
        None,
        None,
        None,
    );

    let main_content = Layout::new(
        Some("MAIN CONTENT\n\nThis is the primary content area.\nIt expands to fill available space.".to_string()),
        Some("main".to_string()),
        None,
        None,
        Some(2),
        None,
    );

    // Body splits into sidebar + main (row split = side by side).
    let mut body = Layout::new(
        None,
        Some("body".to_string()),
        None,
        None,
        None,
        None,
    );
    body.split_row(vec![sidebar, main_content]);

    let footer = Layout::new(
        Some("FOOTER -- status bar".to_string()),
        Some("footer".to_string()),
        Some(3),
        None,
        None,
        None,
    );

    // Root splits into header / body / footer (column split = stacked).
    let mut root = Layout::new(
        None,
        Some("root".to_string()),
        None,
        None,
        None,
        None,
    );
    root.split_column(vec![header, body, footer]);

    console.print(&root);
}
