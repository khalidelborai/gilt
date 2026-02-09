//! Demonstrates gilt's Tree widget — hierarchical file system display.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;
use gilt::tree::Tree;

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("File System Tree"));

    let bold_blue = Style::parse("bold blue").unwrap();
    let default = Style::null();

    // Root: Project/
    let mut tree = Tree::new(Text::new("Project/", bold_blue.clone()))
        .with_guide_style(Style::parse("dim").unwrap());

    // src/
    let src = tree.add(Text::new("src/", bold_blue.clone()));
    src.add(Text::new("main.rs", default.clone()));
    src.add(Text::new("lib.rs", default.clone()));
    {
        let utils = src.add(Text::new("utils/", bold_blue.clone()));
        utils.add(Text::new("helpers.rs", default.clone()));
    }

    // tests/
    let tests = tree.add(Text::new("tests/", bold_blue.clone()));
    tests.add(Text::new("integration.rs", default.clone()));

    // Top-level files
    tree.add(Text::new("Cargo.toml", default.clone()));
    tree.add(Text::new("README.md", default));

    console.print(&tree);
}
