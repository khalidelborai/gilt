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

    #[test]
    fn test_eyre_deeply_chained() {
        #[derive(Debug, thiserror::Error)]
        #[error("level {0}")]
        struct Level(u8, #[source] Option<Box<Level>>);

        // Build a chain 5 levels deep: level 0 -> level 1 -> ... -> level 4
        let mut chain = Level(4, None);
        for i in (0..4).rev() {
            chain = Level(i, Some(Box::new(chain)));
        }

        let handler = GiltEyreHandler;
        let output = format!("{}", DisplayViaDebugHandler(&handler, &chain));
        // All five levels should appear.
        for i in 0..5u8 {
            let needle = format!("level {i}");
            assert!(
                output.contains(&needle),
                "expected '{needle}' in output, got: {output}",
            );
        }
        // Should also contain "Caused by" since there's a source chain.
        assert!(
            output.contains("Caused by"),
            "expected 'Caused by' in output, got: {output}",
        );
    }

    #[test]
    fn test_eyre_with_context() {
        use eyre::WrapErr;

        let _ = install();

        #[derive(Debug, thiserror::Error)]
        #[error("root cause")]
        struct RootCause;

        let report: eyre::Result<()> = Err(eyre::Report::new(RootCause));
        let report = report.wrap_err("added context message");
        let output = format!("{:?}", report.unwrap_err());
        assert!(
            output.contains("added context message"),
            "expected context message in output, got: {output}",
        );
    }

    #[test]
    fn test_eyre_custom_sections() {
        // The GiltEyreHandler renders error chains via the standard source() chain.
        // "Custom sections" in eyre are typically done by wrapping errors with context.
        // Verify the handler can render multiple context layers without panicking.
        use eyre::WrapErr;

        let _ = install();

        #[derive(Debug, thiserror::Error)]
        #[error("base error")]
        struct BaseError;

        let report: eyre::Result<()> = Err(eyre::Report::new(BaseError));
        let report = report
            .wrap_err("section: retry logic")
            .wrap_err("section: network layer");
        let output = format!("{:?}", report.unwrap_err());
        assert!(
            output.contains("network layer"),
            "expected section header in output, got: {output}",
        );
        assert!(
            output.contains("retry logic"),
            "expected section header in output, got: {output}",
        );
    }

    #[test]
    fn test_eyre_display_format() {
        let _ = install();

        #[derive(Debug, thiserror::Error)]
        #[error("display and debug error")]
        struct DualFormatError;

        let report = eyre::Report::new(DualFormatError);
        // Display format (just the top-level message).
        let display_output = format!("{report}");
        assert!(
            display_output.contains("display and debug error"),
            "expected error in Display output, got: {display_output}",
        );
        // Debug format (uses our handler, includes panel formatting).
        let debug_output = format!("{report:?}");
        assert!(
            debug_output.contains("display and debug error"),
            "expected error in Debug output, got: {debug_output}",
        );
        // Debug output should have richer formatting than Display.
        assert!(
            debug_output.len() >= display_output.len(),
            "Debug output should be at least as long as Display output",
        );
    }

    /// Helper to format an error through the GiltEyreHandler's debug method directly,
    /// bypassing the global hook installation (which can conflict across tests).
    struct DisplayViaDebugHandler<'a, E: std::error::Error + 'static>(&'a GiltEyreHandler, &'a E);

    impl<E: std::error::Error + 'static> fmt::Display for DisplayViaDebugHandler<'_, E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use eyre::EyreHandler;
            self.0.debug(self.1, f)
        }
    }
}
