//! File proxy module for capturing stdout/stderr output in Live display.
//!
//! This module provides a `FileProxy` struct that implements `std::io::Write`
//! and buffers output until a newline is received, then sends it to the Live
//! display for rendering above the live content.

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::{Mutex, Weak};

use crate::console::Console;

/// Shared state for the Live display that can receive captured output.
pub trait LiveOutput: Send + Sync {
    /// Add a line of captured output to the buffer.
    fn push_captured_line(&self, line: String, is_stderr: bool);

    /// Get the console for output.
    fn console(&self) -> &Console;
}

/// A proxy writer that captures output and sends it to a Live display.
///
/// This struct buffers output until a newline is received, then sends
/// the complete line to the Live display for rendering. It is used to
/// capture stdout/stderr output and display it within the Live context.
#[derive(Clone)]
pub struct FileProxy {
    /// Weak reference to the Live state for sending captured lines.
    state: Weak<Mutex<dyn LiveOutput>>,
    /// Whether this proxy is for stderr (true) or stdout (false).
    is_stderr: bool,
    /// Buffer for incomplete lines.
    buffer: RefCell<String>,
}

impl FileProxy {
    /// Create a new FileProxy.
    ///
    /// # Arguments
    ///
    /// * `state` - Weak reference to the Live state
    /// * `is_stderr` - Whether this proxy captures stderr (true) or stdout (false)
    pub fn new(state: Weak<Mutex<dyn LiveOutput>>, is_stderr: bool) -> Self {
        Self {
            state,
            is_stderr,
            buffer: RefCell::new(String::new()),
        }
    }

    /// Check if the Live display is still alive.
    pub fn is_alive(&self) -> bool {
        self.state.upgrade().is_some()
    }

    /// Flush the buffer by sending any remaining content.
    fn flush_buffer(&self) -> io::Result<()> {
        let content = self.buffer.borrow().clone();
        if !content.is_empty() {
            self.send_line(&content);
            self.buffer.borrow_mut().clear();
        }
        Ok(())
    }

    /// Send a line to the Live display.
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
        // Convert bytes to string, handling invalid UTF-8
        let text = String::from_utf8_lossy(buf);

        // Process the text line by line
        let mut lines = text.split('\n').peekable();

        while let Some(line) = lines.next() {
            // Append to buffer
            self.buffer.borrow_mut().push_str(line);

            // If there's a next line, we hit a newline - send the buffer
            if lines.peek().is_some() {
                self.flush_buffer()?;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Note: We don't automatically flush the buffer on explicit flush
        // because we want to keep partial lines buffered until a newline
        // or until the proxy is dropped.
        Ok(())
    }
}

impl Drop for FileProxy {
    fn drop(&mut self) {
        // Flush any remaining content when the proxy is dropped
        let _ = self.flush_buffer();
    }
}

/// A handle for stdout/stderr redirection.
///
/// When this handle is dropped, the redirection is automatically restored.
pub struct RedirectHandle {
    /// The FileProxy for stdout, if stdout redirection is enabled.
    _stdout_proxy: Option<FileProxy>,
    /// The FileProxy for stderr, if stderr redirection is enabled.
    _stderr_proxy: Option<FileProxy>,
}

impl RedirectHandle {
    /// Create a new RedirectHandle.
    pub fn new(stdout_proxy: Option<FileProxy>, stderr_proxy: Option<FileProxy>) -> Self {
        Self {
            _stdout_proxy: stdout_proxy,
            _stderr_proxy: stderr_proxy,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;


    struct MockLiveOutput {
        captured: Arc<Mutex<Vec<(String, bool)>>>,
    }

    impl LiveOutput for MockLiveOutput {
        fn push_captured_line(&self, line: String, is_stderr: bool) {
            self.captured.lock().unwrap().push((line, is_stderr));
        }

        fn console(&self) -> &Console {
            unimplemented!("Not needed for tests")
        }
    }

    #[test]
    fn test_file_proxy_buffers_until_newline() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Mutex::new(MockLiveOutput {
            captured: Arc::clone(&captured),
        }));

        let mut proxy = FileProxy::new(Arc::downgrade(&mock) as Weak<Mutex<dyn LiveOutput>>, false);

        // Write partial line
        write!(proxy, "hello").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 0);

        // Write newline
        write!(proxy, " world\n").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 1);
        assert_eq!(captured.lock().unwrap()[0].0, "hello world");
        assert!(!captured.lock().unwrap()[0].1); // is_stderr = false
    }

    #[test]
    fn test_file_proxy_stderr_flag() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Mutex::new(MockLiveOutput {
            captured: Arc::clone(&captured),
        }));

        let mut proxy = FileProxy::new(Arc::downgrade(&mock) as Weak<Mutex<dyn LiveOutput>>, true);

        writeln!(proxy, "error message").unwrap();
        assert_eq!(captured.lock().unwrap().len(), 1);
        assert!(captured.lock().unwrap()[0].1); // is_stderr = true
    }

    #[test]
    fn test_file_proxy_multiple_lines() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Mutex::new(MockLiveOutput {
            captured: Arc::clone(&captured),
        }));

        let mut proxy = FileProxy::new(Arc::downgrade(&mock) as Weak<Mutex<dyn LiveOutput>>, false);

        writeln!(proxy, "line 1").unwrap();
        writeln!(proxy, "line 2").unwrap();
        writeln!(proxy, "line 3").unwrap();

        let lines = captured.lock().unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "line 1");
        assert_eq!(lines[1].0, "line 2");
        assert_eq!(lines[2].0, "line 3");
    }

    #[test]
    fn test_file_proxy_is_alive() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let mock = Arc::new(Mutex::new(MockLiveOutput {
            captured: Arc::clone(&captured),
        }));

        let proxy = FileProxy::new(Arc::downgrade(&mock) as Weak<Mutex<dyn LiveOutput>>, false);
        assert!(proxy.is_alive());

        // Drop the mock to simulate Live being dropped
        drop(mock);
        assert!(!proxy.is_alive());
    }
}
