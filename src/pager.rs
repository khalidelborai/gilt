//! Pager module for displaying content through a system pager.
//!
//! This module provides the [`Pager`] struct, which pipes content through
//! an external pager program (e.g., `less -r`). It mirrors the functionality
//! of Python rich's `Pager` class.

use std::io::Write;
use std::process::{Command, Stdio};

use thiserror::Error;

/// Errors that can occur during pager operations.
#[derive(Error, Debug)]
pub enum PagerError {
    /// An I/O error occurred while communicating with the pager process.
    #[error("pager I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The pager command was not found or could not be parsed.
    #[error("pager command not found: {0}")]
    NotFound(String),
}

/// A pager that pipes content through an external pager program.
///
/// By default the pager command is `less -r`, which enables raw control
/// character passthrough so ANSI escape sequences render correctly.
///
/// # Examples
///
/// ```no_run
/// use gilt::pager::Pager;
///
/// let pager = Pager::new();
/// pager.show("Hello from the pager!").unwrap();
/// ```
///
/// ```no_run
/// use gilt::pager::Pager;
///
/// let pager = Pager::new().with_command("more");
/// pager.show("Hello from more!").unwrap();
/// ```
pub struct Pager {
    /// The pager command string (program and arguments).
    pub command: String,
}

impl Default for Pager {
    fn default() -> Self {
        Self {
            command: "less -r".to_string(),
        }
    }
}

impl Pager {
    /// Creates a new `Pager` with the default command (`less -r`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pager command. This is a builder method that consumes and
    /// returns `self` for method chaining.
    ///
    /// # Arguments
    ///
    /// * `command` - The full pager command string, e.g. `"less -r"` or `"more"`.
    #[must_use]
    pub fn with_command(mut self, command: &str) -> Self {
        self.command = command.to_string();
        self
    }

    /// Splits the command string into the program name and its arguments.
    ///
    /// Returns `Err(PagerError::NotFound)` if the command string is empty.
    fn parse_command(&self) -> Result<(&str, Vec<&str>), PagerError> {
        let mut parts = self.command.split_whitespace();
        let program = parts.next().ok_or_else(|| {
            PagerError::NotFound("empty pager command".to_string())
        })?;
        let args: Vec<&str> = parts.collect();
        Ok((program, args))
    }

    /// Pipes the given content through the pager process.
    ///
    /// The content is written to the pager's stdin, which is then closed.
    /// The method waits for the pager process to exit. A `BrokenPipe` error
    /// (which occurs when the user quits the pager early) is handled
    /// gracefully and does not produce an error.
    ///
    /// # Errors
    ///
    /// Returns [`PagerError::NotFound`] if the command string is empty.
    /// Returns [`PagerError::Io`] if spawning the process or writing to
    /// stdin fails (except for `BrokenPipe`, which is silently ignored).
    pub fn show(&self, content: &str) -> Result<(), PagerError> {
        let (program, args) = self.parse_command()?;

        let mut child = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    PagerError::NotFound(format!(
                        "pager program '{}' not found",
                        program
                    ))
                } else {
                    PagerError::Io(e)
                }
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            match stdin.write_all(content.as_bytes()) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
                Err(e) => return Err(PagerError::Io(e)),
            }
            // stdin is dropped here, closing the pipe
        }

        child.wait()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_command() {
        let pager = Pager::new();
        assert_eq!(pager.command, "less -r");
    }

    #[test]
    fn test_default_trait() {
        let pager = Pager::default();
        assert_eq!(pager.command, "less -r");
    }

    #[test]
    fn test_with_command() {
        let pager = Pager::new().with_command("more");
        assert_eq!(pager.command, "more");
    }

    #[test]
    fn test_with_command_chaining() {
        let pager = Pager::new()
            .with_command("bat --paging=always")
            .with_command("less -R");
        assert_eq!(pager.command, "less -R");
    }

    #[test]
    fn test_parse_command_simple() {
        let pager = Pager::new().with_command("less");
        let (program, args) = pager.parse_command().unwrap();
        assert_eq!(program, "less");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_with_args() {
        let pager = Pager::new().with_command("less -r -X");
        let (program, args) = pager.parse_command().unwrap();
        assert_eq!(program, "less");
        assert_eq!(args, vec!["-r", "-X"]);
    }

    #[test]
    fn test_parse_command_empty() {
        let pager = Pager::new().with_command("");
        let result = pager.parse_command();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PagerError::NotFound(_)));
    }

    #[test]
    fn test_parse_command_whitespace_only() {
        let pager = Pager::new().with_command("   ");
        let result = pager.parse_command();
        assert!(result.is_err());
    }

    #[test]
    fn test_show_empty_command() {
        let pager = Pager::new().with_command("");
        let result = pager.show("hello");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_show_nonexistent_pager() {
        let pager = Pager::new().with_command("this_pager_does_not_exist_xyz");
        let result = pager.show("hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_show_with_echo_cat() {
        // Use `cat` as a pager substitute: it reads stdin and writes to stdout.
        // This verifies the full spawn-write-close-wait pipeline.
        let pager = Pager::new().with_command("cat");
        let result = pager.show("Hello, pager!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_with_true_command() {
        // `true` ignores stdin and exits 0 — may produce BrokenPipe.
        let pager = Pager::new().with_command("true");
        let result = pager.show("ignored content");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_empty_content() {
        let pager = Pager::new().with_command("cat");
        let result = pager.show("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_large_content() {
        let content = "x".repeat(100_000);
        let pager = Pager::new().with_command("cat");
        let result = pager.show(&content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_unicode_content() {
        let pager = Pager::new().with_command("cat");
        let result = pager.show("Hello \u{1F600} world \u{2603} \u{00E9}\u{00E8}\u{00EA}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pager_error_display_io() {
        let io_err = std::io::Error::other("disk full");
        let pager_err = PagerError::Io(io_err);
        assert_eq!(pager_err.to_string(), "pager I/O error: disk full");
    }

    #[test]
    fn test_pager_error_display_not_found() {
        let err = PagerError::NotFound("mycommand".to_string());
        assert_eq!(err.to_string(), "pager command not found: mycommand");
    }

    #[test]
    fn test_pager_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(PagerError::NotFound("test".to_string()));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_pager_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
        let pager_err: PagerError = io_err.into();
        assert!(matches!(pager_err, PagerError::Io(_)));
        assert!(pager_err.to_string().contains("no access"));
    }

    #[test]
    fn test_pager_error_debug() {
        let err = PagerError::NotFound("test_cmd".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("test_cmd"));
    }

    #[test]
    fn test_pager_command_field_public() {
        let mut pager = Pager::new();
        pager.command = "more".to_string();
        assert_eq!(pager.command, "more");
    }
}
