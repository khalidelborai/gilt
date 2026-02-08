//! Integration with the `miette` diagnostic reporting crate.
//!
//! When the `miette` feature is enabled, gilt provides a [`GiltMietteHandler`]
//! that renders diagnostics using gilt's styled terminal output.
//!
//! # Setup
//! ```ignore
//! gilt::miette_handler::install();
//! ```

use crate::console::Console;
use crate::panel::Panel;
use crate::style::Style;
use crate::text::Text;
use miette::{Diagnostic, ReportHandler};
use std::fmt;

/// A miette [`ReportHandler`] that renders diagnostics using gilt's [`Console`].
///
/// The handler formats errors inside a styled [`Panel`] with:
/// - The main error message in bold red
/// - The diagnostic code (if present)
/// - Help text (if present)
/// - A URL link (if present)
/// - The full error source chain
pub struct GiltMietteHandler {
    /// Whether to show the diagnostic code.
    pub show_code: bool,
    /// Whether to show the URL.
    pub show_url: bool,
    /// Whether to show help text.
    pub show_help: bool,
}

impl GiltMietteHandler {
    /// Create a new handler with all display options enabled.
    pub fn new() -> Self {
        Self {
            show_code: true,
            show_url: true,
            show_help: true,
        }
    }
}

impl Default for GiltMietteHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportHandler for GiltMietteHandler {
    fn debug(&self, error: &dyn Diagnostic, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(false)
            .build();
        console.begin_capture();

        // Build markup string for the panel content.
        let title = format!("{error}");
        let mut markup = format!("[bold red]Error:[/bold red] {title}");

        if self.show_code {
            if let Some(code) = error.code() {
                markup.push_str(&format!("\n[dim]Code:[/dim] {code}"));
            }
        }

        if self.show_help {
            if let Some(help) = error.help() {
                markup.push_str(&format!("\n[bold cyan]Help:[/bold cyan] {help}"));
            }
        }

        if self.show_url {
            if let Some(url) = error.url() {
                markup.push_str(&format!("\n[blue underline]{url}[/blue underline]"));
            }
        }

        // Walk the error source chain.
        let mut source = std::error::Error::source(error);
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
        panel.title = Some(Text::new("Diagnostic", Style::null()));

        console.print(&panel);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

/// Install the gilt miette handler as the global error reporter.
///
/// This sets [`GiltMietteHandler`] as the miette report hook. If a hook was
/// already installed, the call is silently ignored.
pub fn install() {
    miette::set_hook(Box::new(|_| Box::new(GiltMietteHandler::new()))).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = GiltMietteHandler::new();
        assert!(handler.show_code);
        assert!(handler.show_url);
        assert!(handler.show_help);
    }

    #[test]
    fn test_handler_default() {
        let handler = GiltMietteHandler::default();
        assert!(handler.show_code);
    }

    #[test]
    fn test_handler_formats_simple_error() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("something went wrong")]
        #[diagnostic(code(gilt::test::error), help("try doing X instead"))]
        struct TestError;

        // Install our handler (may silently fail if already installed).
        install();

        let report = miette::Report::new_boxed(Box::new(TestError));
        // Format via Debug -- this exercises either our handler or the default.
        let output = format!("{report:?}");
        // The output should at minimum contain the error message.
        assert!(
            output.contains("something went wrong"),
            "expected error message in output, got: {output}",
        );
    }

    #[test]
    fn test_handler_formats_chained_error() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("outer error")]
        struct OuterError(#[from] InnerError);

        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("inner error")]
        struct InnerError;

        let outer = OuterError(InnerError);
        let report = miette::Report::new(outer);
        // Verify formatting doesn't panic.
        let output = format!("{report:?}");
        assert!(
            output.contains("outer error"),
            "expected outer error message in output, got: {output}",
        );
    }
}
