//! Text enums for justification and overflow handling.

/// Text justification method for aligning text within a given width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JustifyMethod {
    /// Use the default justification (equivalent to left).
    Default,
    /// Align text to the left edge.
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right edge.
    Right,
    /// Distribute text evenly across the full width by expanding spaces.
    Full,
}

/// Strategy for handling text that exceeds the available width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowMethod {
    /// Wrap overflowing text onto the next line at character boundaries.
    Fold,
    /// Truncate overflowing text silently.
    Crop,
    /// Truncate overflowing text and append an ellipsis character.
    Ellipsis,
    /// Allow text to overflow without any modification.
    Ignore,
}
