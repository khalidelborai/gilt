//! Utility modules for gilt.
//!
//! This module contains various utility types and functions used throughout
//! the gilt library for terminal manipulation, styling, and formatting.

pub mod align_widget;
pub mod ansi;
pub mod bar;
pub mod box_chars;
pub mod cells;
pub mod constrain;
pub mod containers;
pub mod control;
pub mod default_styles;
pub mod diagnose;
pub mod emoji;
pub mod emoji_codes;
pub mod emoji_replace;
pub mod file_proxy;
pub mod filesize;
pub mod group;
pub(crate) mod hash;
pub mod highlighter;
pub mod inspect;
pub mod padding;
pub mod pretty;
pub mod protocol;
pub mod ratio;
pub mod region;
pub mod scope;
pub mod styled;
pub mod styled_str;

// Re-export commonly used items for convenience
pub use align_widget::{Align, HorizontalAlign, VerticalAlign};
pub use ansi::AnsiDecoder;
pub use bar::Bar;
pub use box_chars::{
    BoxChars, ASCII, ASCII2, ASCII_DOUBLE_HEAD, DOUBLE, DOUBLE_EDGE, HEAVY, HEAVY_EDGE, HEAVY_HEAD,
    HORIZONTALS, MARKDOWN, MINIMAL, MINIMAL_DOUBLE_HEAD, MINIMAL_HEAVY_HEAD, ROUNDED, SIMPLE,
    SIMPLE_HEAD, SIMPLE_HEAVY, SQUARE, SQUARE_DOUBLE_HEAD,
};
pub use cells::{
    cell_len, chop_cells, get_character_cell_size, is_single_cell_widths, set_cell_size,
    split_graphemes, split_text,
};
pub use constrain::Constrain;
pub use control::{escape_control_codes, strip_control_codes, Control};
pub use default_styles::DEFAULT_STYLES;
pub use diagnose::{
    print_report, report, ColorSupport, DiagnosticReport, PlatformInfo, TerminalInfo,
    UnicodeSupport,
};
pub use emoji::{Emoji, NoEmoji};
pub use filesize::{binary, decimal, pick_unit_and_suffix};
pub use group::Group;
pub use highlighter::{
    Highlighter, ISO8601Highlighter, JSONHighlighter, NullHighlighter, RegexHighlighter,
    ReprHighlighter,
};
pub use inspect::Inspect;
pub use padding::Padding;
pub use protocol::{
    as_renderable_mut, as_renderable_ref, GiltCast, IntoRenderable, RenderableBox, RenderableExt,
};
pub use ratio::{ratio_distribute, ratio_reduce, ratio_resolve, Edge};
pub use region::Region;
pub use scope::Scope;
pub use styled::Styled;
pub use styled_str::{StyledStr, Stylize};
