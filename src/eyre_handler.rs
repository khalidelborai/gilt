//! Integration with the `eyre` error reporting crate.
//!
//! When the `eyre` feature is enabled, gilt provides a [`GiltEyreHandler`]
//! that renders error reports using gilt's styled terminal output.
//!
//! # Setup
//! ```ignore
//! gilt::eyre_handler::install().expect("failed to install gilt eyre handler");
//! ```

use crate::console::Console;
use crate::panel::Panel;
use crate::style::Style;
use crate::text::Text;
use std::fmt;

/// An eyre [`EyreHandler`](eyre::EyreHandler) that renders error reports
/// using gilt's [`Console`].
///
/// The handler formats errors inside a styled [`Panel`] with:
/// - The main error message in bold red
/// - The full error source chain
pub struct GiltEyreHandler;

impl eyre::EyreHandler for GiltEyreHandler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let width = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(false)
            .build();
        console.begin_capture();

        let title = format!("{error}");
        let mut markup = format!("[bold red]Error:[/bold red] {title}");

        // Walk the error source chain.
        let mut source = error.source();
        if source.is_some() {
            markup.push_str("\n\n[bold]Caused by:[/bold]");
        }
        let mut i = 0;
        while let Some(err) = source {
            markup.push_str(&format!("\n  {i}. {err}"));
            source = err.source();
            i += 1;
        }

        let text = Text::from_markup(&markup).unwrap_or_else(|_| Text::new(&markup, Style::null()));
        let mut panel = Panel::new(text);
        panel.title = Some(Text::new("Error Report", Style::null()));

        console.print(&panel);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

/// Install the gilt eyre handler as the global error reporter.
///
/// # Errors
///
/// Returns [`eyre::InstallError`] if a hook was already installed.
pub fn install() -> Result<(), eyre::InstallError> {
    eyre::set_hook(Box::new(|_| Box::new(GiltEyreHandler)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let _handler = GiltEyreHandler;
    }

    #[test]
    fn test_install_returns_ok_or_already_installed() {
        // First call may succeed or fail if another test already installed a hook.
        // Either way, it should not panic.
        let _ = install();
    }

    #[test]
    fn test_handler_formats_simple_error() {
        // Install our handler (ignore if already installed by another test).
        let _ = install();

        #[derive(Debug, thiserror::Error)]
        #[error("something went wrong")]
        struct TestError;

        let report = eyre::Report::new(TestError);
        let output = format!("{report:?}");
        assert!(
            output.contains("something went wrong"),
            "expected error message in output, got: {output}",
        );
    }

    #[test]
    fn test_handler_formats_chained_error() {
        let _ = install();

        #[derive(Debug, thiserror::Error)]
        #[error("outer error")]
        struct OuterError(#[from] InnerError);

        #[derive(Debug, thiserror::Error)]
        #[error("inner error")]
        struct InnerError;

        let inner = InnerError;
        let outer = OuterError(inner);
        let report = eyre::Report::new(outer);
        let output = format!("{report:?}");
        assert!(
            output.contains("outer error"),
            "expected outer error message in output, got: {output}",
        );
    }
}
