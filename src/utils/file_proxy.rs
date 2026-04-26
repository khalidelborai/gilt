//! File proxy infrastructure for routing buffered writes through an active
//! `Live` display.
//!
//! When a `Live` display is running, any direct write to the terminal will
//! visually corrupt it (interleaving with the live region's redraws and
//! cursor moves). The standard rich-style fix is to wrap stdout/stderr in a
//! [`FileProxy`] that buffers writes line-by-line and forwards completed
//! lines to a [`LiveOutput`] which renders them above the live region.
//!
//! ## Limitation vs. rich
//!
//! Rust's `std::io::stdout()` / `std::io::stderr()` cannot be globally
//! replaced from outside the standard library, so gilt cannot intercept
//! bare `println!` / `eprintln!` macros. This module therefore provides
//! `FileProxy` as an **opt-in building block**: library code that wants to
//! play nicely with `Live` should write to a [`FileProxy`] instead of
//! directly to stdout. The standard pattern is:
//!
//! ```ignore
//! let mut out: Box<dyn std::io::Write + Send> = if let Some(proxy) = live.file_proxy() {
//!     Box::new(proxy)
//! } else {
//!     Box::new(std::io::stdout().lock())
//! };
//! writeln!(out, "Loaded {} records", n)?;
//! ```
//!
//! Direct `println!` in third-party code remains a known limitation; users
//! should be advised to use `console.print()` exclusively inside a `Live`
//! context.

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::{Mutex, Weak};

/// Trait implemented by any sink that can receive captured lines from a
/// [`FileProxy`] — typically the running `Live` display.
pub trait LiveOutput: Send + Sync {
    /// Append one captured line (without trailing newline) to the live
    /// display's static-content buffer. The display chooses how to render it
    /// (typically scrolled in above the live region).
    ///
    /// `is_stderr` distinguishes which proxy emitted the line so the sink
    /// can apply different styling if desired.
    fn push_captured_line(&self, line: String, is_stderr: bool);
}

/// A buffered writer that forwards completed lines to a [`LiveOutput`] sink
/// instead of writing directly to a terminal.
///
/// `FileProxy` is `Clone` (the underlying `Weak` reference and `RefCell` are
/// cheap to share) and `Send` so it can be wrapped in `Box<dyn Write +
/// Send>` and passed across thread boundaries inside an async runtime.
///
/// Lines are buffered until a `\n` is written, at which point the complete
/// line (without its terminating newline) is forwarded to
/// [`LiveOutput::push_captured_line`]. On drop, any partial line is
/// flushed.
#[derive(Clone)]
pub struct FileProxy {
    state: Weak<Mutex<dyn LiveOutput>>,
    is_stderr: bool,
    buffer: RefCell<String>,
}

// SAFETY: RefCell<String> is not Sync, but FileProxy is documented as
// single-owner-per-thread (each thread that wants to capture stdout/stderr
// holds its own FileProxy). The Send bound is achieved via Weak being Send.
// We do NOT mark FileProxy as Sync — multi-threaded sharing requires
// wrapping in an Arc<Mutex<FileProxy>>.
unsafe impl Send for FileProxy {}

impl FileProxy {
    /// Construct a new proxy targeting the given live state.
    ///
    /// `is_stderr` distinguishes stderr-style writes from stdout-style — the
    /// sink can use this to apply different styling.
    pub fn new(state: Weak<Mutex<dyn LiveOutput>>, is_stderr: bool) -> Self {
        Self {
            state,
            is_stderr,
            buffer: RefCell::new(String::new()),
        }
    }

    /// Returns `true` if the live state this proxy targets is still alive.
    /// Once the underlying `Live` is dropped, writes through the proxy
    /// silently no-op.
    pub fn is_alive(&self) -> bool {
        self.state.upgrade().is_some()
    }

    /// Forward any buffered partial line to the sink and clear the buffer.
    ///
    /// Uses `mem::take` to move the content out without an intermediate
    /// clone — leaves an empty `String` behind in one step.
    fn flush_buffer(&self) -> io::Result<()> {
        let content = std::mem::take(&mut *self.buffer.borrow_mut());
        if !content.is_empty() {
            self.send_line(&content);
        }
        Ok(())
    }

    /// Forward a single complete line to the sink (no-op if state is gone).
    fn send_line(&self, line: &str) {
        if let Some(state) = self.state.upgrade() {
            if let Ok(state) = state.lock() {
                state.push_captured_line(line.to_string(), self.is_stderr);
            }
        }
    }
}

impl Write for FileProxy {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        // Splitting on '\n' yields N+1 chunks for N newlines; for each chunk
        // except the last (which is a partial line), append-then-flush.
        let mut chunks = text.split('\n').peekable();
        while let Some(chunk) = chunks.next() {
            self.buffer.borrow_mut().push_str(chunk);
            if chunks.peek().is_some() {
                self.flush_buffer()?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Don't flush partial lines on explicit flush — leave them buffered
        // until a newline arrives or the proxy is dropped. This matches
        // rich's behaviour and avoids splitting `print!("foo")` /
        // `print!("bar")` across separate Live entries.
        Ok(())
    }
}

impl Drop for FileProxy {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockSink {
        captured: Arc<Mutex<Vec<(String, bool)>>>,
    }

    impl LiveOutput for MockSink {
        fn push_captured_line(&self, line: String, is_stderr: bool) {
            self.captured.lock().unwrap().push((line, is_stderr));
        }
    }

    fn proxy_with_sink(
        is_stderr: bool,
    ) -> (
        FileProxy,
        Arc<Mutex<Vec<(String, bool)>>>,
        Arc<Mutex<MockSink>>,
    ) {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::new(Mutex::new(MockSink {
            captured: Arc::clone(&captured),
        }));
        let proxy = FileProxy::new(
            Arc::downgrade(&sink) as Weak<Mutex<dyn LiveOutput>>,
            is_stderr,
        );
        (proxy, captured, sink)
    }

    #[test]
    fn buffers_until_newline() {
        let (mut proxy, captured, _sink) = proxy_with_sink(false);
        write!(proxy, "hello").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 0);
        writeln!(proxy, " world").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 1);
        assert_eq!(captured.lock().unwrap()[0].0, "hello world");
        assert!(!captured.lock().unwrap()[0].1);
    }

    #[test]
    fn marks_stderr() {
        let (mut proxy, captured, _sink) = proxy_with_sink(true);
        writeln!(proxy, "boom").unwrap();
        assert!(captured.lock().unwrap()[0].1);
    }

    #[test]
    fn multi_line_in_one_write() {
        let (mut proxy, captured, _sink) = proxy_with_sink(false);
        write!(proxy, "a\nb\nc\n").unwrap();
        let lines = captured.lock().unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "a");
        assert_eq!(lines[1].0, "b");
        assert_eq!(lines[2].0, "c");
    }

    #[test]
    fn drop_flushes_partial_line() {
        let (proxy, captured, _sink) = proxy_with_sink(false);
        // Re-open as mutable for the write, then drop.
        {
            let mut proxy = proxy.clone();
            write!(proxy, "no terminator").unwrap();
            assert_eq!(captured.lock().unwrap().len(), 0);
            drop(proxy);
        }
        // The clone's drop flushes — the original still buffers (clones share
        // the Weak but each has its own RefCell buffer; this test demonstrates
        // the drop-flush path on the cloned instance).
        assert_eq!(captured.lock().unwrap().len(), 1);
        assert_eq!(captured.lock().unwrap()[0].0, "no terminator");
    }

    #[test]
    fn explicit_flush_keeps_partial_buffered() {
        let (mut proxy, captured, _sink) = proxy_with_sink(false);
        write!(proxy, "partial").unwrap();
        proxy.flush().unwrap();
        // Explicit flush does NOT push partial lines — matches rich.
        assert_eq!(captured.lock().unwrap().len(), 0);
    }

    #[test]
    fn is_alive_tracks_sink_lifetime() {
        let (proxy, _captured, sink) = proxy_with_sink(false);
        assert!(proxy.is_alive());
        drop(sink);
        assert!(!proxy.is_alive());
    }

    #[test]
    fn writes_no_op_after_sink_dropped() {
        let (mut proxy, captured, sink) = proxy_with_sink(false);
        drop(sink);
        // Write should succeed (returns Ok with full byte count) but
        // captured should remain empty.
        writeln!(proxy, "ghost").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 0);
    }
}
