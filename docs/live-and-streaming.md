# Live & streaming guide

`gilt`'s [`Live`](https://docs.rs/gilt/latest/gilt/live/struct.Live.html) display
renders content that updates **in place** — progress dashboards, streaming LLM
output, build pipelines, log tails. Since v1.5.2 a `Live` holds **any
`Renderable`** (not just `Text`) and re-renders it through its own console on
every frame, so the content always reflows at the real terminal width and uses
the console's theme.

## The model

```rust
use gilt::live::Live;
use gilt::text::Text;
use gilt::style::Style;

let mut live = Live::new(Text::new("Working…", Style::null()));
live.start();                                    // hides the cursor, paints
live.update_renderable(Text::new("Done!", Style::null()), true); // repaint now
live.stop();                                     // restores the terminal
```

- `update_renderable(r, refresh)` is **lock-free** — worker threads can push
  updates without ever blocking the renderer. Pass `refresh = true` to repaint
  immediately, or `false` to let the next auto-refresh pick it up.
- A `Live` is `Send + Sync`; share it behind an `Arc` and update from any thread.
- Identical consecutive frames are skipped automatically (no wasted terminal I/O).

## Any renderable, streamed

Because `Live` re-renders the widget each frame, you can stream a growing
document and it reflows correctly:

```rust
use gilt::live::Live;
use gilt::markdown::Markdown;

let mut live = Live::from_renderable(Markdown::new(""));
live.start();
for chunk in stream {                 // e.g. tokens from an LLM
    acc.push_str(chunk);
    live.update_renderable(Markdown::new(&acc), true);
}
live.stop();
```

The same pattern works for `Table`, `Tree`, `Panel`, `Layout`, or your own
`Renderable`. See the runnable examples:

| Example | Widget |
|---|---|
| `cargo run --example live_markdown` | streaming `Markdown` |
| `cargo run --example live_tree` | a build pipeline as a `Tree` |
| `cargo run --example live_status_panel` | a framed `Panel` |
| `cargo run --example top_lite_table` | a live `Table` monitor |

## Printing above the live region

To emit normal scrollback output (logs, results) **above** a running live
display without corrupting it:

```rust
live.print_above(Text::styled("✓ step complete", "green"));
```

The live region is lifted, the content is printed into the scrollback, and the
region is redrawn below it.

## Pausing & resuming

To hand the terminal's bottom row from one live display to another — for example
a sticky footer that needs to step aside while a child display takes over — pause
the outer display, run the child, then resume:

```rust
footer.pause();   // erase the footer in place; keep its content & state
child.start();    // a child Live renders here, with no stale footer above it
child.stop();
footer.resume();  // redraw the footer where the cursor now sits
```

`pause()` halts the background refresh and erases the current render *in place*
(the same erase a transient `stop` performs), but unlike a non-transient `stop`
it emits **no trailing newline**, so the previous frame is not left behind in the
scrollback. The cursor is shown again so the child display behaves normally.
`resume()` re-hides the cursor, redraws the preserved renderable at the cursor's
current position (drawing downward, leaving any output that scrolled in while
paused untouched), and restarts the refresh.

This is cleaner and less error-prone than toggling `transient` around
`stop`/`start` and rebuilding the renderable. `is_paused()` reports the state;
`is_started()` stays `true` while paused. `start`/`stop` behave exactly as
before.

## Vertical overflow

When content is taller than the terminal, choose how it's clipped:

```rust
use gilt::live::Live;
use gilt::live_render::VerticalOverflowMethod;

let live = Live::new(content).with_vertical_overflow(VerticalOverflowMethod::Crop);
//                                                   Crop | Ellipsis | Visible
```

## Transient & alternate-screen

- `with_transient(true)` clears the live region when the display stops.
- The console's `screen_guard()` switches to the alternate screen (and restores
  it on drop) for full-screen apps.

## Cleanup safety

State that must be torn down is guarded by RAII so a `?` early-return or panic
never leaves the terminal in a bad state:

```rust
let mut console = gilt::console::Console::new();
{
    let mut g = console.screen_guard();   // enters alt-screen
    // … if this scope exits early, the alt-screen is exited on drop …
}

{
    let g = console.capture_guard();      // begin_capture()
    // … work …
    let text = g.end();                   // end_capture() -> String
}                                          // (or: dropped -> capture ended safely)
```

## Async (feature `async`)

With the `async` feature, drive a live display from Tokio:

```toml
gilt = { version = "1.6", features = ["async"] }
```

```rust
use gilt::live::Live;
use gilt::r#async::LiveWatchExt;          // brings Live::watch into scope
use tokio::sync::watch;

// 1. Watch a state channel — renders the latest value, coalescing bursts:
let (tx, rx) = watch::channel(0u64);
tokio::spawn(Live::new(initial).watch(rx, |n| Text::from(format!("count: {n}"))));
// `tx.send(..)` from anywhere; the live view follows. Ends when `tx` drops.

// 2. Or drive a Live alongside your own future, cancel-safe:
let result = gilt::r#async::async_run(Live::new(spinner), 20.0, async {
    do_work().await
}).await;
```

`LiveAsync` (and `async_run`/`watch`) are **cancel-safe**: if the future is
dropped — a losing `tokio::select!` branch, a `?` early-return, or a panic — a
synchronous `Drop` aborts the background refresh task and restores the terminal
(cursor shown, alternate screen exited). You never leak terminal state.

## Terminal width & color

- Width comes from the console: with the default `terminal-size` feature, gilt
  detects the real terminal size via `ioctl` (`COLUMNS`/`LINES` env override it);
  on wasm or `--no-default-features`, set it explicitly with
  `Console::builder().width(w)`.
- Color auto-disables when output is **not** a terminal (piped/redirected),
  unless forced with `FORCE_COLOR`/`CLICOLOR_FORCE` or
  `Console::builder().force_terminal(true)`. `NO_COLOR` is always honored.
