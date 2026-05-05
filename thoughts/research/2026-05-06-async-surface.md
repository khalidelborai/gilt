# Async surface audit — gilt v1.1

Scope: what `feature = "async"` exposes today, where sync I/O remains in the render path, what a tokio user would reasonably expect, and whether to invest.

## 1. What's in `async` today

Single file: `src/async.rs` (977 lines). `src/async_rt/` exists but is empty (dead directory).
Re-export: `pub mod r#async;` in `src/lib.rs:792`. Nothing is hoisted into `gilt::` root.

Public surface:

- `ProgressStreamExt` — extension on `futures_core::Stream`; `track_progress(desc, total)` wraps a stream, advances a `Progress` task by 1 per item.
- `ProgressStream<S>` — the wrapper type returned by the ext trait.
- `LiveAsync` — async wrapper around `Live`. Builder, `start/update/stop` are `async fn`. Internally spawns a `tokio::task` that ticks at `refresh_interval` and calls the underlying sync `Live::refresh()` (which itself writes to `stdout`).
- `ProgressChannel` + `ProgressSender` — `mpsc` pair; sender posts `Set(f64)` / `Finish`, receiver `run().await`s, drives a sync `Progress`.
- `async::fs` — `read_with_progress` and `copy_with_progress` over `tokio::fs`, 8KB chunks, drives a sync `Progress` per-call.

Also feature-gated: `http` (= `async + reqwest`) — `RequestBuilderProgress` ext trait for `reqwest::RequestBuilder`.

Shape: **async wrappers around sync sinks**, not an end-to-end async render path.

## 2. Sync I/O in the render path

`Console::write_segments` and `Console::flush_buffer` (`src/console.rs:1091-1095, 1141-1143`) hard-code `std::io::stdout().write_all(...)` + `flush()`. Errors swallowed (`let _ =`). There is **no `with_writer(W)`, no `Box<dyn Write>` field, no AsyncWrite path.** Capture/record divert at the segment level (before the writer); `quiet` shortcuts before write. To get bytes out without stdout you must `begin_capture` and read `end_capture()` as a `String`.

`Live::start` (`src/live/mod.rs:266`) spawns an `std::thread`, not a tokio task. `LiveAsync` wraps this thread-driven `Live` and adds its own tokio interval on top — so a `LiveAsync` actually has an OS thread *and* a tokio task running. Fine for a 4Hz UI, but it's a layered design, not a native one.

`error/traceback.rs:369` writes to `stderr` synchronously.

## 3. Rich (Python) async-relevant primitives

Rich is sync. `Console.print` writes through `self.file` (defaults to `sys.stdout`) — Python rich allows `Console(file=...)` with any `IO[str]`-like, which Rich's tests exploit for capture. Gilt's `Console` does not currently have this writer slot; capture is the only escape valve.

So gilt is *less* swappable than rich on the writer axis, before we even discuss async.

## 4. Realistic gaps a tokio user would hit

1. **`Console::with_writer(W: Write)`** — no swappable writer at all. Blocks piping to a log file, an in-memory buffer outside capture mode, or anything non-stdout.
2. **`AsyncWriteConsole` / `AsyncConsole<W: AsyncWrite>`** — render to a tokio socket, a hyper response body, an SSH channel. Not exotic for ops/devtools CLIs.
3. **`Live::async_run<F: Future>(self, fut)`** — drive Live for the duration of a future, no manual `start()/stop().await`. Today you assemble it from `LiveAsync::start/stop`.
4. **`Progress::from_stream(s, total)` returning `impl Stream`** — the existing `ProgressStreamExt` covers half this; what's missing is the *finished* signal (no `Finish` semantics on stream end vs. `ProgressChannel::Finish`).
5. **`tokio::sync::watch::Receiver<T>` driven Live** — render whatever the latest `T` is, coalescing intermediate updates. Today every `update().await` takes the mutex; under churn you pay per-update.
6. **Cancellation-safety**: `LiveAsync::stop` is not declared cancel-safe; if a caller's `select!` drops the stop future mid-await, the OS terminal may be left in alt-screen / cursor-hidden state (no `Drop` cleanup of the inner `Live` happens because the mutex guard is released between the `state.stopped = true` write and the `state.live.stop()` call).
7. **`http` is the only `reqwest` integration** — no `axum::body::Body` streaming target, no `tonic`, no `hyper`. Not blocking, but worth listing.
8. **Structured async logging sink** — `tracing` feature exists for the formatter side; nothing yet for "render gilt widgets *as* tracing events asynchronously."
9. **`tokio::process::Command` progress wrapper** — `async::fs::read_with_progress` exists; the symmetric `spawn_with_progress` for piping a child process's stdout through a progress bar does not.
10. **No `Send + 'static` audit on public renderables** — needed for any of the above to cross `tokio::spawn` cleanly.

## 5. Concrete small-scope additions

**Pick #1: `Console::with_writer(Box<dyn Write + Send>)`** (~80 LOC + tests). Additive, no API break — default stays `stdout()`. Unblocks gap #1 and is the prerequisite for #2. Suggested signature:

```rust
impl Console {
    pub fn with_writer<W: Write + Send + 'static>(self, w: W) -> Self;
    pub fn writer_mut(&mut self) -> &mut (dyn Write + Send);
}
```

**Pick #2: `Live::async_run`** (~60 LOC, behind `feature = "async"`). Sugar over the existing `LiveAsync::{start, update, stop}` with proper `Drop`-on-cancel cleanup. Suggested signature:

```rust
impl Live {
    pub async fn async_run<F, T>(self, fut: F) -> T
    where F: Future<Output = T>;
}
```

Both land in v1.3 as additive features. ~140 LOC + ~80 LOC of tests.

**Defer**: `AsyncWriteConsole` (gap #2). Real value but requires a parallel render path — `write_segments` is a hot path with `&mut self` semantics; making it async-aware is a refactor, not a feature add. v1.4 or v2.

## 6. Counter — leave it sync

Steel-man for status quo:

- **TUI is single-threaded.** Render path runs on one thread; `stdout` is line-buffered to a TTY whose write throughput dwarfs anything gilt produces. There is no I/O-bound case: 4Hz × 80×24 cells ≈ 8KB/s. Tokio gives us nothing here.
- **`spawn_blocking` is the documented escape hatch.** A tokio app that wants to call `console.print(...)` from inside an async fn calls `tokio::task::spawn_blocking`. README + one example covers 95% of the demand.
- **`LiveAsync` already exists** for the niche where you genuinely want a tokio-driven refresh interval. Adding more async types raises maintenance cost (two render paths to keep correct) for a user base that mostly wants colored text.
- **`AsyncWrite` console is a footgun.** Writes to a network socket can block on backpressure; the render loop was never meant to await. You'd be inviting bugs (deadlock under TCP backpressure) for a use case that's better served by capturing to a string and shipping the string yourself.
- **The strongest signal of demand is issue tracker volume on `r#async`.** Quick check before investing: how many issues mention tokio/async? If <5, pick #1 is the only one worth doing — it's general utility, not an async bet.

## Recommendation

Do **Pick #1** (`with_writer`) regardless of async story — it's been a rich-vs-gilt parity gap since v1.0 and unblocks both async work and ordinary log-to-file. Defer the async-native render path until issue volume justifies it. If you do invest, **Pick #2** (`async_run`) is the cheapest demo of "we take tokio seriously" and the cleanest fix for the cancel-safety bug in #4.6.
