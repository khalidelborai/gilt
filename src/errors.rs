//! Error types used throughout the gilt library.
//!
//! This module provides domain-specific error types for different components
//! of the gilt library, mirroring Python rich's error hierarchy while leveraging
//! Rust's type system for better error handling ergonomics.

use thiserror::Error;

/// Errors that can occur when parsing color specifications.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ColorParseError {
    /// Invalid hexadecimal color format.
    #[error("invalid hex color format: {0}")]
    InvalidHexFormat(String),

    /// Invalid RGB/RGBA color format.
    #[error("invalid RGB color format: {0}")]
    InvalidRgbFormat(String),

    /// Color component value out of valid range.
    #[error("color component out of range: {0}")]
    ComponentOutOfRange(String),

    /// Unknown color name.
    #[error("unknown color name: {0}")]
    UnknownColorName(String),

    /// Invalid color specification.
    #[error("invalid color specification: {0}")]
    InvalidColorSpec(String),
}

/// Errors that can occur when parsing or applying styles.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StyleError {
    /// Invalid style syntax in a style string.
    #[error("invalid style syntax: {0}")]
    InvalidSyntax(String),

    /// Unknown or invalid style attribute.
    #[error("unknown style attribute: {0}")]
    UnknownAttribute(String),

    /// Missing required style definition.
    #[error("missing style definition: {0}")]
    MissingStyle(String),

    /// Invalid combination of style attributes.
    #[error("invalid style combination: {0}")]
    InvalidCombination(String),

    /// Style stack error (push/pop mismatch).
    #[error("style stack error: {0}")]
    StackError(String),
}

/// Errors that can occur during console operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConsoleError {
    /// Error during rendering operation.
    #[error("rendering error: {0}")]
    RenderError(String),

    /// Object cannot be rendered.
    #[error("object is not renderable: {0}")]
    NotRenderable(String),

    /// Invalid markup syntax.
    #[error("invalid markup syntax: {0}")]
    MarkupError(String),

    /// Live display error.
    #[error("live display error: {0}")]
    LiveError(String),

    /// Alternate screen not available.
    #[error("alternate screen not available: {0}")]
    NoAltScreen(String),

    /// Generic console operation error.
    #[error("console error: {0}")]
    Generic(String),
}

/// Errors that can occur during segment operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SegmentError {
    /// Invalid segment construction.
    #[error("invalid segment: {0}")]
    InvalidSegment(String),

    /// Segment processing error.
    #[error("segment processing error: {0}")]
    ProcessingError(String),
}

/// Errors that can occur during cell operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CellError {
    /// Invalid cell width calculation.
    #[error("invalid cell width: {0}")]
    InvalidWidth(String),

    /// Unicode processing error.
    #[error("unicode processing error: {0}")]
    UnicodeError(String),
}

/// Errors that can occur during palette operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum PaletteError {
    /// Invalid palette index.
    #[error("invalid palette index: {0}")]
    InvalidIndex(usize),

    /// Palette not available.
    #[error("palette not available: {0}")]
    NotAvailable(String),
}

/// Errors that can occur when parsing Rich markup.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum MarkupError {
    /// Closing tag doesn't match any open tag.
    #[error("closing tag '{tag}' at position {position} doesn't match any open tag")]
    MismatchedTag { tag: String, position: usize },

    /// Implicit close tag `[/]` has nothing to close.
    #[error("closing tag '[/]' at position {position} has nothing to close")]
    NothingToClose { position: usize },
}

/// A general Result type alias for gilt operations.
pub type GiltResult<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    // ColorParseError tests
    #[test]
    fn test_color_parse_error_invalid_hex() {
        let err = ColorParseError::InvalidHexFormat("#gggggg".to_string());
        assert_eq!(err.to_string(), "invalid hex color format: #gggggg");
    }

    #[test]
    fn test_color_parse_error_invalid_rgb() {
        let err = ColorParseError::InvalidRgbFormat("rgb(300, 0, 0)".to_string());
        assert_eq!(err.to_string(), "invalid RGB color format: rgb(300, 0, 0)");
    }

    #[test]
    fn test_color_parse_error_component_out_of_range() {
        let err = ColorParseError::ComponentOutOfRange("red value 300 exceeds 255".to_string());
        assert_eq!(
            err.to_string(),
            "color component out of range: red value 300 exceeds 255"
        );
    }

    #[test]
    fn test_color_parse_error_unknown_name() {
        let err = ColorParseError::UnknownColorName("notacolor".to_string());
        assert_eq!(err.to_string(), "unknown color name: notacolor");
    }

    #[test]
    fn test_color_parse_error_invalid_spec() {
        let err = ColorParseError::InvalidColorSpec("???".to_string());
        assert_eq!(err.to_string(), "invalid color specification: ???");
    }

    #[test]
    fn test_color_parse_error_clone() {
        let err1 = ColorParseError::InvalidHexFormat("#gg".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_color_parse_error_equality() {
        let err1 = ColorParseError::UnknownColorName("red".to_string());
        let err2 = ColorParseError::UnknownColorName("red".to_string());
        let err3 = ColorParseError::UnknownColorName("blue".to_string());
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_color_parse_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(ColorParseError::InvalidHexFormat("test".to_string()));
        assert!(err.to_string().contains("invalid hex color format"));
    }

    // StyleError tests
    #[test]
    fn test_style_error_invalid_syntax() {
        let err = StyleError::InvalidSyntax("bold on purple red".to_string());
        assert_eq!(err.to_string(), "invalid style syntax: bold on purple red");
    }

    #[test]
    fn test_style_error_unknown_attribute() {
        let err = StyleError::UnknownAttribute("blinking".to_string());
        assert_eq!(err.to_string(), "unknown style attribute: blinking");
    }

    #[test]
    fn test_style_error_missing_style() {
        let err = StyleError::MissingStyle("warning".to_string());
        assert_eq!(err.to_string(), "missing style definition: warning");
    }

    #[test]
    fn test_style_error_invalid_combination() {
        let err = StyleError::InvalidCombination("bold and dim".to_string());
        assert_eq!(err.to_string(), "invalid style combination: bold and dim");
    }

    #[test]
    fn test_style_error_stack_error() {
        let err = StyleError::StackError("pop from empty stack".to_string());
        assert_eq!(err.to_string(), "style stack error: pop from empty stack");
    }

    #[test]
    fn test_style_error_clone() {
        let err1 = StyleError::UnknownAttribute("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_style_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(StyleError::InvalidSyntax("test".to_string()));
        assert!(err.to_string().contains("invalid style syntax"));
    }

    // ConsoleError tests
    #[test]
    fn test_console_error_render_error() {
        let err = ConsoleError::RenderError("width calculation failed".to_string());
        assert_eq!(err.to_string(), "rendering error: width calculation failed");
    }

    #[test]
    fn test_console_error_not_renderable() {
        let err = ConsoleError::NotRenderable("NoneType".to_string());
        assert_eq!(err.to_string(), "object is not renderable: NoneType");
    }

    #[test]
    fn test_console_error_markup_error() {
        let err = ConsoleError::MarkupError("unclosed tag [bold".to_string());
        assert_eq!(err.to_string(), "invalid markup syntax: unclosed tag [bold");
    }

    #[test]
    fn test_console_error_live_error() {
        let err = ConsoleError::LiveError("cannot nest live displays".to_string());
        assert_eq!(
            err.to_string(),
            "live display error: cannot nest live displays"
        );
    }

    #[test]
    fn test_console_error_no_alt_screen() {
        let err =
            ConsoleError::NoAltScreen("terminal does not support alternate screen".to_string());
        assert_eq!(
            err.to_string(),
            "alternate screen not available: terminal does not support alternate screen"
        );
    }

    #[test]
    fn test_console_error_generic() {
        let err = ConsoleError::Generic("unknown error".to_string());
        assert_eq!(err.to_string(), "console error: unknown error");
    }

    #[test]
    fn test_console_error_clone() {
        let err1 = ConsoleError::MarkupError("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_console_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(ConsoleError::NotRenderable("test".to_string()));
        assert!(err.to_string().contains("not renderable"));
    }

    // SegmentError tests
    #[test]
    fn test_segment_error_invalid_segment() {
        let err = SegmentError::InvalidSegment("empty text".to_string());
        assert_eq!(err.to_string(), "invalid segment: empty text");
    }

    #[test]
    fn test_segment_error_processing() {
        let err = SegmentError::ProcessingError("failed to split".to_string());
        assert_eq!(err.to_string(), "segment processing error: failed to split");
    }

    #[test]
    fn test_segment_error_clone() {
        let err1 = SegmentError::InvalidSegment("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_segment_error_is_error_trait() {
        let err: Box<dyn std::error::Error> =
            Box::new(SegmentError::ProcessingError("test".to_string()));
        assert!(err.to_string().contains("processing error"));
    }

    // CellError tests
    #[test]
    fn test_cell_error_invalid_width() {
        let err = CellError::InvalidWidth("negative width".to_string());
        assert_eq!(err.to_string(), "invalid cell width: negative width");
    }

    #[test]
    fn test_cell_error_unicode() {
        let err = CellError::UnicodeError("invalid UTF-8 sequence".to_string());
        assert_eq!(
            err.to_string(),
            "unicode processing error: invalid UTF-8 sequence"
        );
    }

    #[test]
    fn test_cell_error_clone() {
        let err1 = CellError::InvalidWidth("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_cell_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(CellError::UnicodeError("test".to_string()));
        assert!(err.to_string().contains("unicode processing"));
    }

    // PaletteError tests
    #[test]
    fn test_palette_error_invalid_index() {
        let err = PaletteError::InvalidIndex(300);
        assert_eq!(err.to_string(), "invalid palette index: 300");
    }

    #[test]
    fn test_palette_error_not_available() {
        let err = PaletteError::NotAvailable("256-color mode required".to_string());
        assert_eq!(
            err.to_string(),
            "palette not available: 256-color mode required"
        );
    }

    #[test]
    fn test_palette_error_clone() {
        let err1 = PaletteError::InvalidIndex(42);
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_palette_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(PaletteError::InvalidIndex(100));
        assert!(err.to_string().contains("invalid palette index"));
    }

    // Cross-type tests
    #[test]
    fn test_different_error_types_not_equal() {
        // This shouldn't compile if we tried to compare different error types
        // Just verify they can both be boxed as Error trait objects
        let color_err: Box<dyn std::error::Error> =
            Box::new(ColorParseError::InvalidHexFormat("test".to_string()));
        let style_err: Box<dyn std::error::Error> =
            Box::new(StyleError::InvalidSyntax("test".to_string()));

        // Both should have different error messages
        assert_ne!(color_err.to_string(), style_err.to_string());
    }

    #[test]
    fn test_all_errors_implement_debug() {
        // Verify Debug implementation works for all error types
        format!(
            "{:?}",
            ColorParseError::InvalidHexFormat("test".to_string())
        );
        format!("{:?}", StyleError::InvalidSyntax("test".to_string()));
        format!("{:?}", ConsoleError::Generic("test".to_string()));
        format!("{:?}", SegmentError::InvalidSegment("test".to_string()));
        format!("{:?}", CellError::InvalidWidth("test".to_string()));
        format!("{:?}", PaletteError::InvalidIndex(0));
    }

    #[test]
    fn test_gilt_result_type_alias() {
        // Test that GiltResult can be used with different error types
        let result1: GiltResult<i32> = Ok(42);
        assert!(result1.is_ok());

        let result2: GiltResult<String> = Err(Box::new(ColorParseError::InvalidHexFormat(
            "bad".to_string(),
        )));
        assert!(result2.is_err());
    }

    #[test]
    fn test_error_source_chain() {
        // Verify errors can be part of error chains (source method exists)
        let err = ColorParseError::InvalidHexFormat("test".to_string());
        let boxed: Box<dyn std::error::Error> = Box::new(err);
        // source() returns None for our simple errors, but method exists
        assert!(boxed.source().is_none());
    }
}
