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

    #[test]
    #[allow(unused_assignments)]
    fn test_miette_error_with_source_code() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("unexpected token")]
        #[diagnostic(code(gilt::parse::unexpected_token))]
        struct ParseError {
            #[source_code]
            src: miette::NamedSource<String>,
            #[label("here")]
            span: miette::SourceSpan,
        }

        let src = "let x = ;".to_string();
        let err = ParseError {
            src: miette::NamedSource::new("test.rs", src),
            span: (8, 1).into(),
        };

        let handler = GiltMietteHandler::new();
        // Format via the handler's ReportHandler::debug method.
        let output = format!("{}", DisplayViaDebugHandler(&handler, &err));
        assert!(
            output.contains("unexpected token"),
            "expected error message in output, got: {output}",
        );
    }

    #[test]
    fn test_miette_error_with_help() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("file not found")]
        #[diagnostic(help("check that the path exists and is readable"))]
        struct NotFoundError;

        let handler = GiltMietteHandler::new();
        let output = format!("{}", DisplayViaDebugHandler(&handler, &NotFoundError));
        assert!(
            output.contains("file not found"),
            "expected error message in output, got: {output}",
        );
        assert!(
            output.contains("check that the path exists"),
            "expected help text in output, got: {output}",
        );
    }

    #[test]
    fn test_miette_error_with_url() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("deprecated feature used")]
        #[diagnostic(url("https://example.com/docs/deprecation"))]
        struct DeprecationError;

        let handler = GiltMietteHandler::new();
        let output = format!("{}", DisplayViaDebugHandler(&handler, &DeprecationError));
        assert!(
            output.contains("deprecated feature used"),
            "expected error message in output, got: {output}",
        );
        assert!(
            output.contains("https://example.com/docs/deprecation"),
            "expected URL in output, got: {output}",
        );
    }

    #[test]
    #[allow(unused_assignments)]
    fn test_miette_multiple_labels() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("type mismatch")]
        #[diagnostic(code(gilt::check::mismatch))]
        struct TypeMismatch {
            #[source_code]
            src: miette::NamedSource<String>,
            #[label("expected type here")]
            expected_span: miette::SourceSpan,
            #[label("found type here")]
            found_span: miette::SourceSpan,
        }

        let src = "let x: i32 = \"hello\";".to_string();
        let err = TypeMismatch {
            src: miette::NamedSource::new("test.rs", src),
            expected_span: (7, 3).into(),
            found_span: (13, 7).into(),
        };

        let handler = GiltMietteHandler::new();
        let output = format!("{}", DisplayViaDebugHandler(&handler, &err));
        assert!(
            output.contains("type mismatch"),
            "expected error message in output, got: {output}",
        );
    }

    #[test]
    fn test_miette_severity_levels() {
        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("this is a warning")]
        #[diagnostic(severity(Warning))]
        struct WarningDiag;

        #[derive(Debug, thiserror::Error, miette::Diagnostic)]
        #[error("this is advice")]
        #[diagnostic(severity(Advice))]
        struct AdviceDiag;

        let handler = GiltMietteHandler::new();

        let warn_output = format!("{}", DisplayViaDebugHandler(&handler, &WarningDiag));
        assert!(
            warn_output.contains("this is a warning"),
            "expected warning message in output, got: {warn_output}",
        );

        let advice_output = format!("{}", DisplayViaDebugHandler(&handler, &AdviceDiag));
        assert!(
            advice_output.contains("this is advice"),
            "expected advice message in output, got: {advice_output}",
        );
    }

    /// Helper to format a diagnostic through the GiltMietteHandler's debug method.
    struct DisplayViaDebugHandler<'a, E: miette::Diagnostic>(&'a GiltMietteHandler, &'a E);

    impl<E: miette::Diagnostic> fmt::Display for DisplayViaDebugHandler<'_, E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use miette::ReportHandler;
            self.0.debug(self.1, f)
        }
    }
}
