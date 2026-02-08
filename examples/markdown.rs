//! Markdown rendering example -- demonstrates the Markdown widget.
//!
//! Run with: `cargo run --example markdown`

use gilt::console::Console;
use gilt::markdown::Markdown;

fn main() {
    let mut console = Console::builder().width(80).force_terminal(true).build();

    let markup = r#"# Gilt Markdown Demo

## Features

Gilt can render **rich markdown** directly in your terminal, including
*italic text*, **bold text**, and `inline code`.

## Bullet List

- First item with some detail
- Second item showing **bold** inside a list
- Third item with `code` formatting

## Code Block

```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

## Blockquote

> The best way to predict the future is to invent it.
> -- Alan Kay

## Links

Check out [Rust](https://www.rust-lang.org) for more information.
"#;

    let md = Markdown::new(markup);
    console.print(&md);
}
