//! Padding demo. Run: `cargo run --example padding`

use gilt::console::Console;
use gilt::padding::{Padding, PaddingDimensions};
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let test = Padding::new(
        Text::new("Hello", Style::null()),
        PaddingDimensions::Pair(2, 4),
        Style::parse("on blue"),
        false,
    );
    Console::default().print(&test);
}
