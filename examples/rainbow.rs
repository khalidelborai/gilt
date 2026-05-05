//! Rainbow text demo. Run: `cargo run --example rainbow`

use gilt::console::Console;
use gilt::gradient::Gradient;

fn main() {
    Console::default().print(&Gradient::rainbow(
        "I must not fear. Fear is the mind-killer.",
    ));
}
