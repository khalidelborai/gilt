//! Streaming Markdown into a `Live` display — the LLM-token-streaming pattern.
//!
//! Since v1.5.2 a `Live` holds the `Markdown` renderable itself and re-renders
//! it through the live console on every frame, so the document reflows at the
//! real terminal width and uses the console's theme as it grows. This is the
//! faithful equivalent of rich's `live.update(Markdown(...))`.
//!
//! Run with: `cargo run --example live_markdown`

use gilt::live::Live;
use gilt::markdown::Markdown;
use std::thread;
use std::time::Duration;

fn main() {
    // The "response" we reveal incrementally, as if streamed from an LLM.
    let document = "\
# Streaming Markdown

Watch this **render live** as tokens arrive — headings, *emphasis*,
`inline code`, lists, and a fenced code block all reflow at your terminal's
width because the document is re-rendered through the live console each frame.

## Highlights

- Rendered through the **live console** — correct width and theme
- Resize-responsive: the whole document re-lays-out every frame
- One call per chunk: `live.update_renderable(Markdown::new(acc), true)`

## Usage

```rust
let mut live = Live::from_renderable(Markdown::new(\"\"));
live.start();
for chunk in stream {
    acc.push_str(chunk);
    live.update_renderable(Markdown::new(&acc), true);
}
live.stop();
```

Any `Renderable` — `Table`, `Tree`, `Panel`, `Layout` — works the same way.
";

    // Split into word-ish tokens (keeping the trailing space/newline) so the
    // reveal looks like a stream of tokens.
    let tokens: Vec<&str> = document.split_inclusive(char::is_whitespace).collect();

    // Disable auto-refresh: each `update_renderable(_, true)` drives one repaint,
    // so the reveal is deterministic and tied to token arrival.
    let mut live = Live::from_renderable(Markdown::new("")).with_auto_refresh(false);
    live.start();

    let mut acc = String::new();
    for token in tokens {
        acc.push_str(token);
        // Re-render the growing document; `true` forces an immediate repaint.
        live.update_renderable(Markdown::new(&acc), true);
        thread::sleep(Duration::from_millis(25));
    }

    // Hold the final frame briefly, then restore the terminal.
    thread::sleep(Duration::from_millis(800));
    live.stop();
}
