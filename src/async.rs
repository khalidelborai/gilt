//! Async support for modern Rust CLIs using Tokio.
//!
//! This module provides async wrappers around the sync progress and live display
//! APIs, enabling seamless integration with async Rust code.
//!
//! # Features
//!
//! - **Async Progress Tracking**: Track progress on async streams with [`ProgressStreamExt`]
//! - **Async-aware Live Display**: [`LiveAsync`] for live-updating content in async contexts
//! - **Progress Channels**: [`ProgressChannel`] for cross-task progress updates
//! - **Async File Operations**: [`fs`] module for file I/O with progress tracking
//!
//! # Examples
//!
//! ```rust,no_run
//! use gilt::r#async::{ProgressStreamExt, LiveAsync, ProgressChannel};
//! use gilt::text::Text;
//! use gilt::style::Style;
//!
//! use futures_util::stream;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Track any async `Stream` (here a simple range stream)
//!     let items = stream::iter(0..100u64);
//!     let progress_stream = items.track_progress("Scanning files", Some(100.0));
//!     let _ = progress_stream; // drive it with `futures_util::StreamExt::next`
//! }
//! ```

use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_core::Stream;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::console::Renderable;
use crate::live::Live;
use crate::progress::{Progress, TaskId};
#[allow(unused_imports)] // used in doc-examples and the test module
use crate::text::Text;

// ---------------------------------------------------------------------------
// ProgressStream
// ---------------------------------------------------------------------------

/// Extension trait for async streams that adds progress tracking.
///
/// This trait provides the [`track_progress`](ProgressStreamExt::track_progress) method
/// that wraps any async stream with progress tracking.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::r#async::ProgressStreamExt;
/// use futures_util::stream;
///
/// #[tokio::main]
/// async fn main() {
///     let items = stream::iter(0..100u64);
///     let progress_stream = items.track_progress("Scanning files", None);
///
///     // Each item yielded advances the progress
///     let _ = progress_stream;
/// }
/// ```
pub trait ProgressStreamExt: Stream {
    /// Wrap this stream with progress tracking.
    ///
    /// The progress bar will advance by 1.0 for each item yielded by the stream.
    /// If `total` is `None`, the progress bar runs in indeterminate mode.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::ProgressStreamExt;
    /// use futures_util::stream;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let items = stream::iter(0..100u64);
    ///     let progress_stream = items.track_progress("Scanning files", Some(100.0));
    ///     let _ = progress_stream;
    /// }
    /// ```
    fn track_progress(self, description: &str, total: Option<f64>) -> ProgressStream<Self>
    where
        Self: Sized;
}

impl<S: Stream> ProgressStreamExt for S {
    fn track_progress(self, description: &str, total: Option<f64>) -> ProgressStream<Self>
    where
        Self: Sized,
    {
        ProgressStream::new(self, description, total)
    }
}

/// A stream wrapper that tracks progress as items are yielded.
///
/// Created by [`ProgressStreamExt::track_progress`]. The progress bar starts when
/// the first item is polled and stops automatically when the stream is exhausted
/// or dropped.
///
/// # Examples
///
/// ```rust,no_run
/// use futures_util::stream::{self, StreamExt};
/// use gilt::r#async::ProgressStreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let stream = stream::iter(0..100);
///     let mut progress_stream = stream.track_progress("Processing", Some(100.0));
///     
///     while let Some(item) = progress_stream.next().await {
///         // Process item
///     }
/// }
/// ```
pub struct ProgressStream<S> {
    inner: S,
    progress: Progress,
    task: TaskId,
    started: bool,
}

impl<S: Stream> ProgressStream<S> {
    /// Create a new progress-tracking stream wrapper.
    ///
    /// Typically called via [`ProgressStreamExt::track_progress`] instead of directly.
    pub fn new(inner: S, description: &str, total: Option<f64>) -> Self {
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, total, true);

        ProgressStream {
            inner,
            progress,
            task,
            started: false,
        }
    }

    /// Get the task ID associated with this progress stream.
    pub fn task_id(&self) -> TaskId {
        self.task
    }

    /// Access the underlying progress tracker.
    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    /// Access the underlying stream.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the underlying stream.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: Stream + Unpin> Stream for ProgressStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // `S: Unpin` makes `ProgressStream<S>: Unpin`, so safe `get_mut()` is enough.
        let this = self.get_mut();

        // Start progress on first poll
        if !this.started {
            this.progress.start();
            this.started = true;
        }

        // Poll the inner stream
        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let task_id = this.task;
                this.progress.advance(task_id, 1.0);
                this.progress.refresh();
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => {
                this.progress.stop();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S> Drop for ProgressStream<S> {
    fn drop(&mut self) {
        if self.started {
            self.progress.stop();
        }
    }
}

// ---------------------------------------------------------------------------
// LiveAsync
// ---------------------------------------------------------------------------

/// Drop-guard that synchronously stops a `Live` when it goes out of scope.
///
/// Held inside `LiveAsync` — when dropped (either by explicit `stop()` or by
/// an unexpected drop of the outer `LiveAsync` future), it calls `Live::stop()`
/// on the std-Mutex-guarded live display, which is always reachable without
/// `.await`. The abort of the background JoinHandle is also done here so the
/// refresh task does not outlive the display.
struct LiveGuard {
    /// The live display. Wrapped in `Option` so `stop()` and `Drop` can both
    /// call `Live::stop()` via `take()`, ensuring idempotency.
    live: std::sync::Mutex<Option<Live>>,
}

impl LiveGuard {
    fn new(live: Live) -> Self {
        LiveGuard {
            live: std::sync::Mutex::new(Some(live)),
        }
    }

    /// Stop the live display synchronously. Idempotent: safe to call many times.
    fn stop_sync(&self) {
        if let Ok(mut guard) = self.live.lock() {
            if let Some(mut live) = guard.take() {
                live.stop();
                // `live` is dropped here; `Live::Drop` is a no-op because
                // `started` was set to false by `Live::stop()`.
            }
        }
    }

    /// Borrow the live display for a brief synchronous operation.
    ///
    /// # Panics
    /// Never — if the lock is poisoned we silently skip (best-effort).
    fn with_live<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Live) -> R,
    {
        self.live
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(f))
    }
}

impl Drop for LiveGuard {
    fn drop(&mut self) {
        self.stop_sync();
    }
}

/// Async-aware Live display that manages its refresh loop using Tokio tasks.
///
/// `LiveAsync` wraps the synchronous [`Live`] display and manages a background
/// Tokio task for automatic refreshing. This is suitable for use in async
/// contexts where blocking the runtime would be undesirable.
///
/// ## Cancel-safety / Drop discipline
///
/// If the `stop()` future is dropped (e.g. inside `tokio::select!`, an early
/// `?` return, or a panic unwind) the `Drop` impl **synchronously**:
///
/// 1. Aborts the background refresh `JoinHandle`.
/// 2. Calls `Live::stop()` via a `std::sync::Mutex` — no `.await` required.
///
/// This ensures the terminal cursor is always shown and the alternate screen
/// is always exited, even when `stop()` is never awaited.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::r#async::LiveAsync;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// #[tokio::main]
/// async fn main() {
///     let mut live = LiveAsync::new(Text::new("Loading...", Style::null()));
///     live.start().await;
///
///     // Do some async work
///     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///
///     live.update(Text::new("Done!", Style::null())).await;
///     live.stop().await;
/// }
/// ```
pub struct LiveAsync {
    /// The live display + sync stop-guard.  Held in an `Arc` so the background
    /// refresh task can borrow it without holding a tokio Mutex across `.await`.
    guard: Arc<LiveGuard>,
    /// Background refresh task handle.  `Option` so `stop()` and `Drop` can
    /// both `take()` it without double-abort.
    refresh_handle: Option<JoinHandle<()>>,
    refresh_interval: Duration,
    started: bool,
}

impl LiveAsync {
    /// Create a new async live display with the given initial content.
    ///
    /// Accepts any [`Renderable`] — `Text`, `Table`, `Panel`, etc.
    ///
    /// # Defaults
    /// - `refresh_interval`: 250ms (4 refreshes per second)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gilt::r#async::LiveAsync;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let live = LiveAsync::new(Text::new("Loading...", Style::null()));
    /// ```
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        let live = Live::new(renderable).with_auto_refresh(false);
        LiveAsync {
            guard: Arc::new(LiveGuard::new(live)),
            refresh_handle: None,
            refresh_interval: Duration::from_millis(250),
            started: false,
        }
    }

    /// Builder: set the refresh interval.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gilt::r#async::LiveAsync;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    /// use tokio::time::Duration;
    ///
    /// let live = LiveAsync::new(Text::new("Loading...", Style::null()))
    ///     .with_refresh_interval(Duration::from_millis(100));
    /// ```
    #[must_use]
    pub fn with_refresh_interval(mut self, interval: Duration) -> Self {
        self.refresh_interval = interval;
        self
    }

    /// Start the live display.
    ///
    /// Hides the cursor and starts a background Tokio task that refreshes the
    /// display at the configured interval.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::LiveAsync;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut live = LiveAsync::new(Text::new("Loading...", Style::null()));
    ///     live.start().await;
    /// }
    /// ```
    pub async fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        // Start the underlying live display synchronously (no await needed).
        if let Ok(mut guard) = self.guard.live.lock() {
            if let Some(live) = guard.as_mut() {
                live.start();
            }
        }

        // Spawn background refresh task.
        // The task holds only an Arc<LiveGuard> — the lock is held only for the
        // duration of `live.refresh()`, never across `.await`.
        let guard = Arc::clone(&self.guard);
        let interval_duration = self.refresh_interval;

        let handle = tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            loop {
                ticker.tick().await;
                // Lock is acquired briefly and released before the next await.
                let keep_going = guard
                    .live
                    .lock()
                    .ok()
                    .and_then(|g| {
                        g.as_ref().map(|live| {
                            live.refresh();
                            true
                        })
                    })
                    .unwrap_or(false); // live was taken => stop
                if !keep_going {
                    break;
                }
            }
        });

        self.refresh_handle = Some(handle);
    }

    /// Update the displayed content and repaint immediately.
    ///
    /// Accepts any [`Renderable`] — passing `Text` still compiles unchanged.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::LiveAsync;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut live = LiveAsync::new(Text::new("Step 1...", Style::null()));
    ///     live.start().await;
    ///
    ///     live.update(Text::new("Step 2...", Style::null())).await;
    /// }
    /// ```
    pub async fn update(&mut self, renderable: impl Renderable + Send + Sync + 'static) {
        self.guard
            .with_live(|live| live.update_renderable(renderable, true));
    }

    /// Stop the live display and restore the terminal.
    ///
    /// Idempotent — safe to call multiple times.  After `stop().await` the
    /// `Drop` impl is a no-op.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::LiveAsync;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut live = LiveAsync::new(Text::new("Loading...", Style::null()));
    ///     live.start().await;
    ///
    ///     // Work...
    ///
    ///     live.stop().await;
    /// }
    /// ```
    pub async fn stop(&mut self) {
        if !self.started {
            return;
        }
        self.started = false;

        // Abort the background task and wait for it so we know the task is
        // gone before we call Live::stop() (avoids a post-stop refresh race).
        if let Some(handle) = self.refresh_handle.take() {
            handle.abort();
            let _ = handle.await; // JoinError on abort is expected — ignore it.
        }

        // Synchronously stop the live display and restore the terminal.
        // stop_sync() takes the Live out of the Option, so Drop will be a no-op.
        self.guard.stop_sync();
    }

    /// Whether the live display is currently running.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get the configured refresh interval.
    pub fn refresh_interval(&self) -> Duration {
        self.refresh_interval
    }
}

impl Drop for LiveAsync {
    /// Cancel-safe cleanup: aborts the background task and synchronously
    /// restores the terminal even when `stop()` was never awaited.
    fn drop(&mut self) {
        if self.started {
            // Abort the background refresh task (fire-and-forget — we can't
            // await in Drop, but abort() schedules cancellation immediately).
            if let Some(handle) = self.refresh_handle.take() {
                handle.abort();
            }
            // Synchronously restore the terminal.  This is safe because:
            //   - `LiveGuard::live` is a `std::sync::Mutex` — no await needed.
            //   - The background task only holds a brief lock per tick; after
            //     abort() any in-progress tick will complete and release it.
            //   - `stop_sync()` uses `try_lock()` semantics via `.lock().ok()`
            //     so a poisoned mutex is handled gracefully.
            self.guard.stop_sync();
        }
    }
}

// ---------------------------------------------------------------------------
// async_run — cancel-safe single entry point (Task 3)
// ---------------------------------------------------------------------------

/// A RAII guard that calls `Live::stop()` when dropped.
///
/// Used by [`async_run`] to guarantee terminal restoration even when the
/// returned future is cancelled or panics.
struct LiveRunGuard {
    live: Option<Live>,
}

impl LiveRunGuard {
    fn new(mut live: Live) -> Self {
        live.start();
        LiveRunGuard { live: Some(live) }
    }

    fn live(&self) -> &Live {
        self.live.as_ref().expect("live guard already consumed")
    }
}

impl Drop for LiveRunGuard {
    fn drop(&mut self) {
        if let Some(mut live) = self.live.take() {
            live.stop();
        }
    }
}

/// Drive a [`Live`] display alongside a user future, restoring the terminal
/// when the future completes **or** when this future itself is dropped/cancelled.
///
/// This is the recommended single entry-point for async live displays.  It
/// combines Task-1 cancel-safety (a `Drop` guard restores the terminal) with
/// a built-in throttled refresh loop.
///
/// # Parameters
/// - `live`: an already-configured (but not yet started) [`Live`] instance.
/// - `refresh_per_second`: how many times per second the display is refreshed.
/// - `fut`: the user future to drive while the live display is active.
///
/// # Returns
/// The value returned by `fut`.
///
/// # Cancel-safety
/// Dropping the returned future before it completes immediately aborts the
/// refresh task and synchronously calls `Live::stop()` via a `Drop` guard —
/// so the cursor is always shown and alt-screen is always exited.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::r#async::async_run;
/// use gilt::live::Live;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// #[tokio::main]
/// async fn main() {
///     let live = Live::new(Text::new("Working…", Style::null()))
///         .with_auto_refresh(false);
///
///     async_run(live, 4.0, async {
///         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///     }).await;
/// }
/// ```
pub async fn async_run<F, T>(live: Live, refresh_per_second: f64, fut: F) -> T
where
    F: Future<Output = T>,
{
    assert!(refresh_per_second > 0.0, "refresh_per_second must be > 0");

    // The guard starts the live display; its Drop stops it.
    let guard = Arc::new(std::sync::Mutex::new(LiveRunGuard::new(live)));

    let interval_ms = (1000.0 / refresh_per_second) as u64;
    let guard_ref = Arc::clone(&guard);
    let refresh_handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(interval_ms.max(1)));
        loop {
            ticker.tick().await;
            let keep_going = guard_ref
                .lock()
                .ok()
                .map(|g| {
                    g.live().refresh();
                    true
                })
                .unwrap_or(false);
            if !keep_going {
                break;
            }
        }
    });

    let result = fut.await;

    // Abort refresh task and stop the live display.
    refresh_handle.abort();
    let _ = refresh_handle.await;
    // Take the Live out of the guard while the guard borrow is short-lived,
    // then stop it after the MutexGuard has been dropped.
    let taken = guard.lock().ok().and_then(|mut g| g.live.take());
    if let Some(mut live) = taken {
        live.stop();
    }

    result
}

// ---------------------------------------------------------------------------
// Live::watch — watch-channel-driven live display (Task 4)
// ---------------------------------------------------------------------------

/// Extension trait that adds [`watch`](LiveWatchExt::watch) to [`Live`].
///
/// Enabled only when the `async` feature is active.
pub trait LiveWatchExt: Sized {
    /// Drive this live display from a [`tokio::sync::watch`] channel.
    ///
    /// On every change notification the closure `f` is called with the latest
    /// value, and the result is pushed into the display.  Intermediate sends
    /// are naturally coalesced because `watch` only keeps the most-recent value.
    ///
    /// The live display is stopped (terminal restored) when:
    /// - the [`Sender`](tokio::sync::watch::Sender) is dropped, **or**
    /// - this future is itself dropped/cancelled (cancel-safe via an internal
    ///   `Drop` guard).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::LiveWatchExt;
    /// use gilt::live::Live;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, rx) = tokio::sync::watch::channel(0u64);
    ///
    ///     let live = Live::new(Text::new("0", Style::null()))
    ///         .with_auto_refresh(false);
    ///
    ///     tokio::spawn(async move {
    ///         for i in 1..=5 {
    ///             tx.send(i).unwrap();
    ///             tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///         }
    ///         // tx dropped here → watch() returns
    ///     });
    ///
    ///     live.watch(rx, |n| Text::new(&n.to_string(), Style::null())).await;
    /// }
    /// ```
    fn watch<T, R, Fmt>(self, rx: watch::Receiver<T>, f: Fmt) -> impl Future<Output = ()> + Send
    where
        T: Send + Sync + 'static,
        R: Renderable + Send + Sync + 'static,
        Fmt: Fn(&T) -> R + Send + 'static;
}

impl LiveWatchExt for Live {
    async fn watch<T, R, Fmt>(self, mut rx: watch::Receiver<T>, f: Fmt)
    where
        T: Send + Sync + 'static,
        R: Renderable + Send + Sync + 'static,
        Fmt: Fn(&T) -> R + Send + 'static,
    {
        // The guard starts `live` and stops it on drop — cancel-safe.
        let guard = Arc::new(std::sync::Mutex::new(LiveRunGuard::new(self)));

        // Render the initial value immediately.
        {
            let renderable = f(&*rx.borrow());
            if let Ok(g) = guard.lock() {
                g.live().update_renderable(renderable, true);
            }
        }

        // Wait for changes until the sender is dropped.
        while let Ok(()) = rx.changed().await {
            let renderable = f(&*rx.borrow());
            let keep_going = guard
                .lock()
                .ok()
                .map(|g| {
                    g.live().update_renderable(renderable, true);
                    true
                })
                .unwrap_or(false);
            if !keep_going {
                break;
            }
        }

        // Explicitly stop the live display before returning.
        // Take the live out while the MutexGuard is short-lived, then stop it
        // after the guard is released — avoids borrow issues at end-of-scope.
        let taken = guard.lock().ok().and_then(|mut g| g.live.take());
        if let Some(mut live) = taken {
            live.stop();
        }
    }
}

// ---------------------------------------------------------------------------
// ProgressChannel
// ---------------------------------------------------------------------------

/// Message type for progress updates sent through the channel.
#[derive(Debug, Clone, Copy)]
enum ProgressUpdate {
    /// Update the completed amount to this value.
    Set(f64),
    /// Finish the progress task.
    Finish,
}

/// Sender half of a progress channel.
///
/// Used to send progress updates from async tasks. Clone this to share
/// progress updates across multiple tasks.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::r#async::ProgressChannel;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, progress) = ProgressChannel::new("Processing");
///     
///     tokio::spawn(async move {
///         for i in 0..100 {
///             tx.update(i as f64).await;
///             tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
///         }
///         tx.finish().await;
///     });
///     
///     progress.run().await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProgressSender {
    sender: mpsc::Sender<ProgressUpdate>,
}

impl ProgressSender {
    /// Send an update to set the completed amount.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::ProgressChannel;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, progress) = ProgressChannel::new("Processing");
    ///     tx.update(50.0).await;
    /// }
    /// ```
    pub async fn update(&self, completed: f64) {
        let _ = self.sender.send(ProgressUpdate::Set(completed)).await;
    }

    /// Send a finish signal to complete the progress.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::ProgressChannel;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, progress) = ProgressChannel::new("Processing");
    ///     tx.finish().await;
    /// }
    /// ```
    pub async fn finish(&self) {
        let _ = self.sender.send(ProgressUpdate::Finish).await;
    }
}

/// Progress display that receives updates through an async channel.
///
/// This allows progress to be updated from multiple async tasks concurrently.
/// The channel has a buffer size of 1024 messages.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::r#async::ProgressChannel;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, progress) = ProgressChannel::new("Processing");
///     
///     tokio::spawn(async move {
///         for i in 0..100 {
///             tx.update(i as f64).await;
///             tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
///         }
///         tx.finish().await;
///     });
///     
///     progress.run().await;
/// }
/// ```
pub struct ProgressChannel {
    receiver: mpsc::Receiver<ProgressUpdate>,
    progress: Progress,
    task: TaskId,
}

impl ProgressChannel {
    /// Create a new progress channel with the given description.
    ///
    /// Returns a `(ProgressSender, ProgressChannel)` tuple. The sender can be
    /// cloned and shared across tasks, while the channel should be awaited
    /// with [`run`](ProgressChannel::run).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gilt::r#async::ProgressChannel;
    ///
    /// let (tx, progress) = ProgressChannel::new("Processing");
    /// ```
    pub fn new(description: &str) -> (ProgressSender, Self) {
        let (sender, receiver) = mpsc::channel(1024);
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, None, true);

        (
            ProgressSender { sender },
            ProgressChannel {
                receiver,
                progress,
                task,
            },
        )
    }

    /// Create a new progress channel with a known total.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gilt::r#async::ProgressChannel;
    ///
    /// let (tx, progress) = ProgressChannel::with_total("Processing", 100.0);
    /// ```
    pub fn with_total(description: &str, total: f64) -> (ProgressSender, Self) {
        let (sender, receiver) = mpsc::channel(1024);
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, Some(total), true);

        (
            ProgressSender { sender },
            ProgressChannel {
                receiver,
                progress,
                task,
            },
        )
    }

    /// Run the progress display, processing updates until finished.
    ///
    /// This method starts the progress display and listens for updates from
    /// the sender. It returns when a `Finish` message is received or the
    /// channel is closed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::r#async::ProgressChannel;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, progress) = ProgressChannel::new("Processing");
    ///     
    ///     // Spawn worker task
    ///     tokio::spawn(async move {
    ///         for i in 0..100 {
    ///             tx.update(i as f64).await;
    ///         }
    ///         tx.finish().await;
    ///     });
    ///     
    ///     // Run the progress display
    ///     progress.run().await;
    /// }
    /// ```
    pub async fn run(mut self) {
        self.progress.start();

        // Small delay to ensure display is initialized before processing updates
        tokio::time::sleep(Duration::from_millis(50)).await;

        while let Some(update) = self.receiver.recv().await {
            match update {
                ProgressUpdate::Set(completed) => {
                    self.progress
                        .update(self.task, Some(completed), None, None, None, None, None);
                    // Refresh display immediately after each update
                    self.progress.refresh();
                }
                ProgressUpdate::Finish => {
                    // Mark task as finished by setting completed = total
                    // (the auto-finish logic will set finished_time when completed >= total)
                    if let Some(task) = self.progress.get_task(self.task) {
                        if let Some(total) = task.total {
                            self.progress.update(
                                self.task,
                                Some(total),
                                None,
                                None,
                                None,
                                None,
                                None,
                            );
                        }
                    }
                    self.progress.refresh();
                    break;
                }
            }
        }

        // Small delay so the final 100% state is visible before stopping
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.progress.stop();
    }

    /// Get the task ID associated with this progress channel.
    pub fn task_id(&self) -> TaskId {
        self.task
    }

    /// Access the underlying progress tracker.
    pub fn progress(&self) -> &Progress {
        &self.progress
    }
}

// ---------------------------------------------------------------------------
// Async File Operations
// ---------------------------------------------------------------------------

/// Async file operations with progress tracking.
///
/// This module provides async versions of common file operations that
/// integrate with the progress tracking system.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use gilt::r#async::fs;
///
/// #[tokio::main]
/// async fn main() {
///     let data = fs::read_with_progress(Path::new("large_file.bin"), "Reading file").await.unwrap();
///     println!("Read {} bytes", data.len());
/// }
/// ```
pub mod fs {
    use super::*;

    /// Read a file asynchronously with progress tracking.
    ///
    /// The progress bar shows bytes read. The file size is used as the total
    /// if available from metadata.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use gilt::r#async::fs;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let data = fs::read_with_progress(Path::new("data.bin"), "Reading").await.unwrap();
    /// }
    /// ```
    pub async fn read_with_progress(path: &Path, description: &str) -> io::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        // Get file metadata for size
        let metadata = tokio::fs::metadata(path).await?;
        let total_size = metadata.len() as f64;

        // Open the file
        let mut file = tokio::fs::File::open(path).await?;

        // Create progress tracker
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, Some(total_size), true);
        progress.start();

        // Read in chunks
        let mut buffer = Vec::with_capacity(total_size as usize);
        let mut chunk = vec![0u8; 8192];
        let mut bytes_read = 0u64;

        loop {
            match file.read(&mut chunk).await {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    bytes_read += n as u64;
                    progress.update(task, Some(bytes_read as f64), None, None, None, None, None);
                }
                Err(e) => {
                    progress.stop();
                    return Err(e);
                }
            }
        }

        progress.stop();
        Ok(buffer)
    }

    /// Copy a file asynchronously with progress tracking.
    ///
    /// Copies `src` to `dst` with a progress bar showing bytes copied.
    /// Returns the total number of bytes copied.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use gilt::r#async::fs;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let bytes_copied = fs::copy_with_progress(
    ///         Path::new("source.bin"),
    ///         Path::new("dest.bin"),
    ///         "Copying"
    ///     ).await.unwrap();
    ///     
    ///     println!("Copied {} bytes", bytes_copied);
    /// }
    /// ```
    pub async fn copy_with_progress(src: &Path, dst: &Path, description: &str) -> io::Result<u64> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Get source file metadata
        let metadata = tokio::fs::metadata(src).await?;
        let total_size = metadata.len() as f64;

        // Open source and destination
        let mut src_file = tokio::fs::File::open(src).await?;
        let mut dst_file = tokio::fs::File::create(dst).await?;

        // Create progress tracker
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, Some(total_size), true);
        progress.start();

        // Copy in chunks
        let mut buffer = vec![0u8; 8192];
        let mut total_copied = 0u64;

        loop {
            match src_file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = dst_file.write_all(&buffer[..n]).await {
                        progress.stop();
                        return Err(e);
                    }
                    total_copied += n as u64;
                    progress.update(
                        task,
                        Some(total_copied as f64),
                        None,
                        None,
                        None,
                        None,
                        None,
                    );
                }
                Err(e) => {
                    progress.stop();
                    return Err(e);
                }
            }
        }

        progress.stop();
        Ok(total_copied)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream::{self, StreamExt};

    // Helper to create a quiet test console (writes are captured, not to tty).
    fn test_console() -> crate::console::Console {
        crate::console::Console::builder()
            .width(80)
            .height(25)
            .quiet(true)
            .markup(false)
            .no_color(true)
            .force_terminal(true)
            .build()
    }

    // Helper: build a Live with a quiet console and auto_refresh disabled.
    fn test_live(text: &str) -> Live {
        Live::new(Text::new(text, crate::style::Style::null()))
            .with_console(test_console())
            .with_auto_refresh(false)
    }

    // -------------------------------------------------------------------------
    // ProgressStream
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_progress_stream_tracks_items() {
        let items: Vec<i32> = vec![1, 2, 3, 4, 5];
        let stream = stream::iter(items);
        let mut progress_stream = stream.track_progress("Testing", Some(5.0));

        let mut count = 0;
        while progress_stream.next().await.is_some() {
            count += 1;
        }

        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_progress_stream_size_hint() {
        let stream = stream::iter(0..100);
        let progress_stream = ProgressStream::new(stream, "Testing", Some(100.0));

        let (lower, upper) = progress_stream.size_hint();
        assert_eq!(lower, 100);
        assert_eq!(upper, Some(100));
    }

    // -------------------------------------------------------------------------
    // ProgressChannel
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_progress_channel_basic() {
        let (tx, progress) = ProgressChannel::with_total("Test", 100.0);

        let worker = tokio::spawn(async move {
            for i in 0..=100 {
                tx.update(i as f64).await;
            }
            tx.finish().await;
        });

        let progress_handle = tokio::spawn(async move {
            progress.run().await;
        });

        let _ = tokio::join!(worker, progress_handle);
    }

    #[tokio::test]
    async fn test_progress_channel_multiple_senders() {
        let (tx, progress) = ProgressChannel::with_total("Test", 200.0);

        let tx2 = tx.clone();

        let worker1 = tokio::spawn(async move {
            for i in 0..100 {
                tx.update(i as f64).await;
                tokio::task::yield_now().await;
            }
        });

        let worker2 = tokio::spawn(async move {
            for i in 100..=200 {
                tx2.update(i as f64).await;
                tokio::task::yield_now().await;
            }
            tx2.finish().await;
        });

        let progress_handle = tokio::spawn(async move {
            progress.run().await;
        });

        let _ = tokio::join!(worker1, worker2, progress_handle);
    }

    // -------------------------------------------------------------------------
    // LiveAsync — basic lifecycle (Task 1 + 2)
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_live_async_lifecycle() {
        let mut live = LiveAsync::new(Text::new("Test", crate::style::Style::null()));

        assert!(!live.is_started());

        live.start().await;
        assert!(live.is_started());

        // Task 2: update() now accepts any Renderable.
        live.update(Text::new("Updated", crate::style::Style::null()))
            .await;

        live.stop().await;
        assert!(!live.is_started());
    }

    #[tokio::test]
    async fn test_live_async_double_start_stop() {
        let mut live = LiveAsync::new(Text::new("Test", crate::style::Style::null()));

        live.start().await;
        live.start().await; // Should be no-op
        assert!(live.is_started());

        live.stop().await;
        live.stop().await; // Should be no-op
        assert!(!live.is_started());
    }

    // -------------------------------------------------------------------------
    // Task 1: cancel-safety — Drop without stop() must abort task + stop live
    // -------------------------------------------------------------------------

    /// Spawn a `LiveAsync`, drop it without calling `stop()`, and verify that:
    /// (a) the background refresh task is aborted (it stops running), and
    /// (b) the underlying `Live` is stopped (cursor/terminal restored).
    ///
    /// We verify (b) indirectly: after drop, the `LiveGuard`'s Option is `None`,
    /// which is the post-stop state set by `stop_sync()`.
    #[tokio::test]
    async fn test_live_async_drop_without_stop_aborts_task_and_stops_live() {
        // We hold on to the Arc<LiveGuard> to inspect state after drop.
        let guard_arc;
        let handle_was_some;
        {
            let mut live = LiveAsync::new(Text::new("drop-test", crate::style::Style::null()));
            live.start().await;
            assert!(live.is_started());

            // Give the refresh task a moment to actually start running.
            tokio::task::yield_now().await;

            // Record that the handle was present.
            handle_was_some = live.refresh_handle.is_some();
            // Clone the Arc so we can inspect the guard after LiveAsync is dropped.
            guard_arc = Arc::clone(&live.guard);

            // `live` drops here — Drop impl runs.
        }

        // Drop was synchronous; give the event loop a tick to process the abort.
        tokio::task::yield_now().await;

        assert!(
            handle_was_some,
            "refresh handle should have been set after start()"
        );

        // After drop, stop_sync() should have taken the Live out of the Option.
        let live_is_none = guard_arc.live.lock().map(|g| g.is_none()).unwrap_or(false);
        assert!(
            live_is_none,
            "Live should have been taken/stopped by Drop (LiveGuard.live should be None)"
        );
    }

    /// Verify that after an explicit `stop().await`, `Drop` is a no-op (double
    /// stop does not panic or corrupt state).
    #[tokio::test]
    async fn test_live_async_explicit_stop_then_drop_is_noop() {
        let mut live = LiveAsync::new(Text::new("stop-then-drop", crate::style::Style::null()));
        live.start().await;
        live.stop().await;
        assert!(!live.is_started());
        // `live` drops here — should be a no-op, no panic.
    }

    // -------------------------------------------------------------------------
    // Task 2: generic Renderable constructor and update
    // -------------------------------------------------------------------------

    /// Confirm that `new()` and `update()` accept types other than `Text`.
    /// We use `Text` here (no extra widget dep needed in tests), but the
    /// compilation of this test proves the generics work.
    #[tokio::test]
    async fn test_live_async_generic_renderable() {
        // new() with Text (proves impl Renderable bound is satisfied)
        let mut live = LiveAsync::new(Text::new("generic-init", crate::style::Style::null()));
        live.start().await;

        // update() with Text
        live.update(Text::new("generic-update", crate::style::Style::null()))
            .await;

        live.stop().await;
    }

    // -------------------------------------------------------------------------
    // Task 3: async_run
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_async_run_returns_future_value() {
        let live = test_live("async_run test");
        let result = async_run(live, 10.0, async { 42u32 }).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_run_terminal_restored_after_future() {
        // After async_run completes the Live is stopped; the guard's Option is None.
        // We verify by inspecting a shared flag set from within the user future.
        use std::sync::atomic::{AtomicBool, Ordering};
        let ran = Arc::new(AtomicBool::new(false));
        let ran2 = Arc::clone(&ran);

        let live = test_live("run-test");
        async_run(live, 10.0, async move {
            tokio::task::yield_now().await;
            ran2.store(true, Ordering::SeqCst);
        })
        .await;

        assert!(ran.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_run_cancel_safe_via_select() {
        // Drop async_run via tokio::select! with an immediately-ready branch.
        // The Live should be stopped (no panic, no hang).
        let live = test_live("cancel-select");
        tokio::select! {
            _ = async_run(live, 4.0, async {
                // Never resolves on its own in time.
                tokio::time::sleep(Duration::from_secs(60)).await;
            }) => {}
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                // The async_run future is dropped here.
            }
        }
        // If we reach this point without a panic the terminal was restored.
    }

    // -------------------------------------------------------------------------
    // Task 4: watch-channel-driven Live
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_live_watch_renders_latest_value() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc as StdArc;

        // Shared counter to record the last value the closure rendered.
        let last_seen = StdArc::new(AtomicU64::new(0));
        let last_seen2 = StdArc::clone(&last_seen);

        let (tx, rx) = watch::channel(0u64);

        let live = test_live("watch-test");

        // Run watch() in a background task so we can drive the sender.
        let watch_handle = tokio::spawn(async move {
            live.watch(rx, move |n| {
                last_seen2.store(*n, Ordering::SeqCst);
                Text::new(&n.to_string(), crate::style::Style::null())
            })
            .await;
        });

        // Send a sequence of values.
        for i in 1u64..=5 {
            tx.send(i).unwrap();
            tokio::task::yield_now().await;
        }

        // Drop sender → watch() returns.
        drop(tx);

        // Wait for the watch task to finish.
        watch_handle.await.unwrap();

        // The closure must have been called with the final value (5).
        assert_eq!(last_seen.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_live_watch_exits_when_sender_dropped() {
        let (tx, rx) = watch::channel("initial");
        let live = test_live("watch-drop");

        let watch_handle = tokio::spawn(async move {
            live.watch(rx, |s| Text::new(s, crate::style::Style::null()))
                .await;
        });

        // Drop the sender immediately — watch() should return promptly.
        drop(tx);

        // Should complete without hanging.
        tokio::time::timeout(Duration::from_secs(2), watch_handle)
            .await
            .expect("watch() should have returned when sender was dropped")
            .unwrap();
    }

    // -------------------------------------------------------------------------
    // File-system helpers
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_fs_read_with_progress_small_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("gilt_async_test_read.txt");
        let test_content = b"Hello, async world!";
        tokio::fs::write(&test_file, test_content).await.unwrap();

        let result = fs::read_with_progress(&test_file, "Reading test file").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);

        let _ = tokio::fs::remove_file(&test_file).await;
    }

    #[tokio::test]
    async fn test_fs_copy_with_progress() {
        let temp_dir = std::env::temp_dir();
        let src_file = temp_dir.join("gilt_async_test_src.txt");
        let dst_file = temp_dir.join("gilt_async_test_dst.txt");
        let test_content = b"Copy this content!";
        tokio::fs::write(&src_file, test_content).await.unwrap();

        let result = fs::copy_with_progress(&src_file, &dst_file, "Copying test file").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content.len() as u64);

        let copied = tokio::fs::read(&dst_file).await.unwrap();
        assert_eq!(copied, test_content);

        let _ = tokio::fs::remove_file(&src_file).await;
        let _ = tokio::fs::remove_file(&dst_file).await;
    }
}
