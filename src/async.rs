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
//! #[tokio::main]
//! async fn main() {
//!     // Track async stream
//!     let stream = tokio::fs::read_dir("./src").await.unwrap();
//!     let progress_stream = stream.track_progress("Scanning files", None);
//!     
//!     // Use the stream
//! }
//! ```

use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_core::Stream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::live::Live;
use crate::progress::{Progress, TaskId};
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
///
/// #[tokio::main]
/// async fn main() {
///     let stream = tokio::fs::read_dir("./src").await.unwrap();
///     let progress_stream = stream.track_progress("Scanning files", None);
///     
///     // Each item yielded advances the progress
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
    ///
    /// #[tokio::main]
    /// async fn main() {
///     let stream = tokio::fs::read_dir("./src").await.unwrap();
    ///     let progress_stream = stream.track_progress("Scanning files", Some(100.0));
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
/// use futures::stream::{self, StreamExt};
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
        let task = progress.add_task(description, total);

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
        // SAFETY: We're not moving out of self, just getting mutable access to fields
        let this = unsafe { self.get_unchecked_mut() };

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

/// Shared state for the LiveAsync background refresh task.
struct LiveAsyncState {
    live: Live,
    stopped: bool,
}

/// Async-aware Live display that manages its refresh loop using Tokio tasks.
///
/// `LiveAsync` wraps the synchronous [`Live`] display and manages a background
/// Tokio task for automatic refreshing. This is suitable for use in async
/// contexts where blocking the runtime would be undesirable.
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
    state: Arc<Mutex<LiveAsyncState>>,
    refresh_handle: Option<JoinHandle<()>>,
    refresh_interval: Duration,
    started: bool,
}

impl LiveAsync {
    /// Create a new async live display with the given initial content.
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
    pub fn new(renderable: Text) -> Self {
        LiveAsync {
            state: Arc::new(Mutex::new(LiveAsyncState {
                live: Live::new(renderable).with_auto_refresh(false),
                stopped: false,
            })),
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
    /// This shows the initial content and starts a background Tokio task
    /// that refreshes the display at the configured interval.
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

        // Start the underlying live display
        {
            let mut state = self.state.lock().await;
            state.live.start();
            state.stopped = false;
        }

        // Spawn background refresh task
        let state = Arc::clone(&self.state);
        let interval_duration = self.refresh_interval;

        let handle = tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            loop {
                ticker.tick().await;

                let state = state.lock().await;
                if state.stopped {
                    break;
                }
                state.live.refresh();
            }
        });

        self.refresh_handle = Some(handle);
    }

    /// Update the displayed content.
    ///
    /// The display is refreshed immediately after the update.
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
    pub async fn update(&mut self, renderable: Text) {
        let mut state = self.state.lock().await;
        state.live.update_renderable(renderable, true);
    }

    /// Stop the live display.
    ///
    /// This stops the background refresh task and restores the terminal state.
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

        // Signal the refresh task to stop
        {
            let mut state = self.state.lock().await;
            state.stopped = true;
        }

        // Abort and wait for the refresh task
        if let Some(handle) = self.refresh_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        // Stop the underlying live display - release lock after
        let mut state = self.state.lock().await;
        state.live.stop();
        // Lock released here when state goes out of scope
    }

    /// Check if the live display is currently running.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get the refresh interval.
    pub fn refresh_interval(&self) -> Duration {
        self.refresh_interval
    }
}

impl Drop for LiveAsync {
    fn drop(&mut self) {
        if self.started {
            // Abort the refresh task
            if let Some(handle) = self.refresh_handle.take() {
                handle.abort();
            }
            // Note: We can't await the task in Drop, but the underlying Live::stop()
            // will be called when Live is dropped. This may leave a stale display
            // line, so calling stop().await explicitly is recommended.
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
        let task = progress.add_task(description, None);

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
        let task = progress.add_task(description, Some(total));

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
                    self.progress.update(
                        self.task,
                        Some(completed),
                        None,
                        None,
                        None,
                        None,
                    );
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
        let task = progress.add_task(description, Some(total_size));
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
                    progress.update(task, Some(bytes_read as f64), None, None, None, None);
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
    pub async fn copy_with_progress(
        src: &Path,
        dst: &Path,
        description: &str,
    ) -> io::Result<u64> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Get source file metadata
        let metadata = tokio::fs::metadata(src).await?;
        let total_size = metadata.len() as f64;

        // Open source and destination
        let mut src_file = tokio::fs::File::open(src).await?;
        let mut dst_file = tokio::fs::File::create(dst).await?;

        // Create progress tracker
        let mut progress = Progress::new(Progress::default_columns()).with_auto_refresh(true);
        let task = progress.add_task(description, Some(total_size));
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
                    progress.update(task, Some(total_copied as f64), None, None, None, None);
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
    use futures::stream::{self, StreamExt};

    // Helper to create a test console
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

    #[tokio::test]
    async fn test_progress_stream_tracks_items() {
        let items: Vec<i32> = vec![1, 2, 3, 4, 5];
        let stream = stream::iter(items);
        let mut progress_stream = stream.track_progress("Testing", Some(5.0));

        let mut count = 0;
        while let Some(_) = progress_stream.next().await {
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

    #[tokio::test]
    async fn test_progress_channel_basic() {
        let (tx, progress) = ProgressChannel::with_total("Test", 100.0);

        let worker = tokio::spawn(async move {
            for i in 0..=100 {
                tx.update(i as f64).await;
            }
            tx.finish().await;
        });

        // Run progress in background
        let progress_handle = tokio::spawn(async move {
            progress.run().await;
        });

        // Wait for both to complete
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

    #[tokio::test]
    async fn test_live_async_lifecycle() {
        let mut live = LiveAsync::new(Text::new("Test", crate::style::Style::null()));

        assert!(!live.is_started());

        live.start().await;
        assert!(live.is_started());

        live.update(Text::new("Updated", crate::style::Style::null())).await;

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

    #[tokio::test]
    async fn test_fs_read_with_progress_small_file() {
        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("gilt_async_test_read.txt");
        let test_content = b"Hello, async world!";
        tokio::fs::write(&test_file, test_content).await.unwrap();

        let result = fs::read_with_progress(&test_file, "Reading test file").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);

        // Cleanup
        let _ = tokio::fs::remove_file(&test_file).await;
    }

    #[tokio::test]
    async fn test_fs_copy_with_progress() {
        // Create source file
        let temp_dir = std::env::temp_dir();
        let src_file = temp_dir.join("gilt_async_test_src.txt");
        let dst_file = temp_dir.join("gilt_async_test_dst.txt");
        let test_content = b"Copy this content!";
        tokio::fs::write(&src_file, test_content).await.unwrap();

        let result = fs::copy_with_progress(&src_file, &dst_file, "Copying test file").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content.len() as u64);

        // Verify content
        let copied = tokio::fs::read(&dst_file).await.unwrap();
        assert_eq!(copied, test_content);

        // Cleanup
        let _ = tokio::fs::remove_file(&src_file).await;
        let _ = tokio::fs::remove_file(&dst_file).await;
    }
}
