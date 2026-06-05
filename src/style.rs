//! Style representation and manipulation for terminal text.
//!
//! This module provides the Style type that represents the visual appearance
//! of terminal text, including colors, text attributes (bold, italic, etc.),
//! and hyperlinks.

use crate::color::{blend_rgb, Color, ColorSystem};
use crate::error::StyleError;
use crate::terminal_theme::TerminalTheme;
use std::fmt;
use std::fmt::Write as _;
use std::ops::Add;

/// Bit positions for text attributes.
const BOLD: u16 = 1 << 0;
const DIM: u16 = 1 << 1;
const ITALIC: u16 = 1 << 2;
const UNDERLINE: u16 = 1 << 3;
const BLINK: u16 = 1 << 4;
const BLINK2: u16 = 1 << 5;
const REVERSE: u16 = 1 << 6;
const CONCEAL: u16 = 1 << 7;
const STRIKE: u16 = 1 << 8;
const UNDERLINE2: u16 = 1 << 9;
const FRAME: u16 = 1 << 10;
const ENCIRCLE: u16 = 1 << 11;
const OVERLINE: u16 = 1 << 12;

/// Underline style variants for extended underline rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnderlineStyle {
    /// Standard single underline (SGR 4)
    Single,
    /// Double underline (SGR 21)
    Double,
    /// Curly/wavy underline (SGR 4:3)
    Curly,
    /// Dotted underline (SGR 4:4)
    Dotted,
    /// Dashed underline (SGR 4:5)
    Dashed,
}

/// A terminal text style with colors, attributes, and links.
#[derive(Clone, Debug)]
pub struct Style {
    /// Foreground color
    color: Option<Color>,
    /// Background color
    bgcolor: Option<Color>,
    /// Bit field of which attributes are set
    set_attributes: u16,
    /// Bit field of attribute values
    attributes: u16,
    /// Optional hyperlink URL
    link: Option<String>,
    /// Optional underline color (SGR 58)
    underline_color: Option<Color>,
    /// Optional underline style variant
    underline_style: Option<UnderlineStyle>,
}

impl Style {
    /// Creates a new style with specified attributes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        color: Option<&str>,
        bgcolor: Option<&str>,
        bold: Option<bool>,
        dim: Option<bool>,
        italic: Option<bool>,
        underline: Option<bool>,
        blink: Option<bool>,
        blink2: Option<bool>,
        reverse: Option<bool>,
        conceal: Option<bool>,
        strike: Option<bool>,
        underline2: Option<bool>,
        frame: Option<bool>,
        encircle: Option<bool>,
        overline: Option<bool>,
        link: Option<&str>,
    ) -> Result<Self, StyleError> {
        let mut style = Style {
            color: None,
            bgcolor: None,
            set_attributes: 0,
            attributes: 0,
            link: None,
            underline_color: None,
            underline_style: None,
        };

        if let Some(c) = color {
            style.color = Some(
                Color::parse(c)
                    .map_err(|e| StyleError::InvalidSyntax(format!("invalid color: {}", e)))?,
            );
        }

        if let Some(bg) = bgcolor {
            style.bgcolor = Some(
                Color::parse(bg)
                    .map_err(|e| StyleError::InvalidSyntax(format!("invalid bgcolor: {}", e)))?,
            );
        }

        style.set_attribute(BOLD, bold);
        style.set_attribute(DIM, dim);
        style.set_attribute(ITALIC, italic);
        style.set_attribute(UNDERLINE, underline);
        style.set_attribute(BLINK, blink);
        style.set_attribute(BLINK2, blink2);
        style.set_attribute(REVERSE, reverse);
        style.set_attribute(CONCEAL, conceal);
        style.set_attribute(STRIKE, strike);
        style.set_attribute(UNDERLINE2, underline2);
        style.set_attribute(FRAME, frame);
        style.set_attribute(ENCIRCLE, encircle);
        style.set_attribute(OVERLINE, overline);

        if let Some(l) = link {
            style.link = Some(l.to_string());
        }

        Ok(style)
    }

    /// Creates an empty null style with no attributes set.
    pub fn null() -> Self {
        Style {
            color: None,
            bgcolor: None,
            set_attributes: 0,
            attributes: 0,
            link: None,
            underline_color: None,
            underline_style: None,
        }
    }

    /// Creates a style from optional colors.
    pub fn from_color(color: Option<Color>, bgcolor: Option<Color>) -> Self {
        Style {
            color,
            bgcolor,
            set_attributes: 0,
            attributes: 0,
            link: None,
            underline_color: None,
            underline_style: None,
        }
    }

    /// Parses a style definition string. **Lossy**: returns
    /// [`Style::null`] on any parse error.
    ///
    /// This is the recommended entry point for the common case of literal
    /// style strings (`"bold red"`, `"on blue"`, …) where a parse failure
    /// indicates a programming bug, not a recoverable runtime condition.
    /// For static literals there is nothing meaningful to do with an error
    /// at the callsite — the lossy form removes the boilerplate
    /// `unwrap_or_else(|_| Style::null())` that wrapped almost every
    /// previous use.
    ///
    /// # When to use [`parse_strict`](Self::parse_strict) instead
    ///
    /// - You're parsing user-supplied style strings (config files, CLI
    ///   flags) and want to surface a syntax error.
    /// - You want to write a unit test that verifies a particular input
    ///   *fails* to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::style::Style;
    ///
    /// let bold_red = Style::parse("bold red");          // no `?`, no `.unwrap()`
    /// let same     = Style::parse("BOLD RED");          // case-insensitive keywords
    /// let null     = Style::parse("not a real style");  // returns Style::null(), no panic
    /// assert!(null.is_null());
    /// ```
    pub fn parse(definition: &str) -> Self {
        Self::parse_strict(definition).unwrap_or_else(|_| Style::null())
    }

    /// Parses a style definition string. **Strict**: returns the parse
    /// error if the input is malformed.
    ///
    /// Prefer [`parse`](Self::parse) for the common case where a parse
    /// failure indicates a programming bug, not a recoverable runtime
    /// condition.
    ///
    /// # Grammar
    /// - Words are split by whitespace
    /// - `"on"` keyword: next word is background color
    /// - `"not"` keyword: next word is attribute name, set to false
    /// - `"link"` keyword: next word is URL
    /// - Known attribute names with aliases
    /// - Anything else: try as foreground color
    pub fn parse_strict(definition: &str) -> Result<Self, StyleError> {
        // Normalise the cache key to lowercase so "BOLD" and "bold" share
        // a single entry.
        let key = definition.to_lowercase();

        // Single lock scope: check cache, parse if missing, insert, drop.
        let mut cache = get_style_cache();
        if let Some(ref mut c) = *cache {
            if let Some(style) = c.get(key.as_str()) {
                return Ok(style.clone());
            }
        }

        // Parse (cache lock still held — recompute-and-insert is correct
        // under contention; no invariant requires exactly-once parsing).
        let style = Self::parse_internal(definition)?;

        if let Some(ref mut c) = *cache {
            c.put(key, style.clone());
        }

        Ok(style)
    }

    /// Internal parsing logic without caching.
    fn parse_internal(definition: &str) -> Result<Self, StyleError> {
        let definition = definition.trim();
        if definition.is_empty() || definition.eq_ignore_ascii_case("none") {
            return Ok(Style::null());
        }

        let mut style = Style::null();
        let words: Vec<&str> = definition.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i].to_lowercase();

            match word.as_str() {
                "on" => {
                    i += 1;
                    if i >= words.len() {
                        return Err(StyleError::InvalidSyntax(
                            "expected color after 'on'".to_string(),
                        ));
                    }
                    let bgcolor_str = words[i];
                    style.bgcolor = Some(Color::parse(bgcolor_str).map_err(|e| {
                        StyleError::InvalidSyntax(format!("invalid background color: {}", e))
                    })?);
                }
                "not" => {
                    i += 1;
                    if i >= words.len() {
                        return Err(StyleError::InvalidSyntax(
                            "expected attribute after 'not'".to_string(),
                        ));
                    }
                    let attr = words[i].to_lowercase();
                    if let Some(bit) = parse_attribute_name(&attr) {
                        style.set_attribute(bit, Some(false));
                    } else {
                        return Err(StyleError::UnknownAttribute(attr));
                    }
                }
                "link" => {
                    i += 1;
                    if i >= words.len() {
                        return Err(StyleError::InvalidSyntax(
                            "expected URL after 'link'".to_string(),
                        ));
                    }
                    style.link = Some(words[i].to_string());
                }
                _ => {
                    // Handle link=URL syntax (use original word to preserve URL case)
                    if word.starts_with("link=") {
                        let url = &words[i]["link=".len()..];
                        if url.is_empty() {
                            return Err(StyleError::InvalidSyntax(
                                "expected URL after 'link='".to_string(),
                            ));
                        }
                        style.link = Some(url.to_string());
                    } else if let Some(ul_style) = parse_underline_style_name(&word) {
                        // Underline style variant (from Display round-trip)
                        style.underline_style = Some(ul_style);
                    } else if word.starts_with("underline_color(") && word.ends_with(')') {
                        // underline_color(<name>) token emitted by Display
                        let inner = &word["underline_color(".len()..word.len() - 1];
                        let color = Color::parse(inner).map_err(|e| {
                            StyleError::InvalidSyntax(format!("invalid underline_color: {}", e))
                        })?;
                        style.underline_color = Some(color);
                    } else if let Some(bit) = parse_attribute_name(&word) {
                        // Try as attribute name
                        style.set_attribute(bit, Some(true));
                    } else {
                        // Try as foreground color
                        match Color::parse(&word) {
                            Ok(color) => style.color = Some(color),
                            Err(e) => {
                                return Err(StyleError::InvalidSyntax(format!(
                                    "unknown attribute or color '{}': {}",
                                    word, e
                                )))
                            }
                        }
                    }
                }
            }

            i += 1;
        }

        Ok(style)
    }

    /// Sets an attribute bit.
    fn set_attribute(&mut self, bit: u16, value: Option<bool>) {
        if let Some(val) = value {
            self.set_attributes |= bit;
            if val {
                self.attributes |= bit;
            } else {
                self.attributes &= !bit;
            }
        }
    }

    /// Gets an attribute value.
    fn get_attribute(&self, bit: u16) -> Option<bool> {
        if self.set_attributes & bit != 0 {
            Some(self.attributes & bit != 0)
        } else {
            None
        }
    }

    /// Returns the bold attribute.
    pub fn bold(&self) -> Option<bool> {
        self.get_attribute(BOLD)
    }

    /// Returns the dim attribute.
    pub fn dim(&self) -> Option<bool> {
        self.get_attribute(DIM)
    }

    /// Returns the italic attribute.
    pub fn italic(&self) -> Option<bool> {
        self.get_attribute(ITALIC)
    }

    /// Returns the underline attribute.
    pub fn underline(&self) -> Option<bool> {
        self.get_attribute(UNDERLINE)
    }

    /// Returns the blink attribute.
    pub fn blink(&self) -> Option<bool> {
        self.get_attribute(BLINK)
    }

    /// Returns the blink2 attribute.
    pub fn blink2(&self) -> Option<bool> {
        self.get_attribute(BLINK2)
    }

    /// Returns the reverse attribute.
    pub fn reverse(&self) -> Option<bool> {
        self.get_attribute(REVERSE)
    }

    /// Returns the conceal attribute.
    pub fn conceal(&self) -> Option<bool> {
        self.get_attribute(CONCEAL)
    }

    /// Returns the strike attribute.
    pub fn strike(&self) -> Option<bool> {
        self.get_attribute(STRIKE)
    }

    /// Returns the underline2 attribute.
    pub fn underline2(&self) -> Option<bool> {
        self.get_attribute(UNDERLINE2)
    }

    /// Returns the frame attribute.
    pub fn frame(&self) -> Option<bool> {
        self.get_attribute(FRAME)
    }

    /// Returns the encircle attribute.
    pub fn encircle(&self) -> Option<bool> {
        self.get_attribute(ENCIRCLE)
    }

    /// Returns the overline attribute.
    pub fn overline(&self) -> Option<bool> {
        self.get_attribute(OVERLINE)
    }

    /// Returns the foreground color.
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Returns the background color.
    pub fn bgcolor(&self) -> Option<&Color> {
        self.bgcolor.as_ref()
    }

    /// Returns the link URL.
    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// Returns the underline color.
    pub fn underline_color(&self) -> Option<&Color> {
        self.underline_color.as_ref()
    }

    /// Returns the underline style.
    pub fn underline_style(&self) -> Option<UnderlineStyle> {
        self.underline_style
    }

    /// Sets the bold attribute.
    pub fn set_bold(&mut self, value: Option<bool>) {
        self.set_attribute(BOLD, value);
    }

    /// Sets the dim attribute.
    pub fn set_dim(&mut self, value: Option<bool>) {
        self.set_attribute(DIM, value);
    }

    /// Sets the italic attribute.
    pub fn set_italic(&mut self, value: Option<bool>) {
        self.set_attribute(ITALIC, value);
    }

    /// Sets the underline attribute.
    pub fn set_underline(&mut self, value: Option<bool>) {
        self.set_attribute(UNDERLINE, value);
    }

    /// Sets the blink attribute.
    pub fn set_blink(&mut self, value: Option<bool>) {
        self.set_attribute(BLINK, value);
    }

    /// Sets the reverse attribute.
    pub fn set_reverse(&mut self, value: Option<bool>) {
        self.set_attribute(REVERSE, value);
    }

    /// Sets the conceal attribute.
    pub fn set_conceal(&mut self, value: Option<bool>) {
        self.set_attribute(CONCEAL, value);
    }

    /// Sets the strike attribute.
    pub fn set_strike(&mut self, value: Option<bool>) {
        self.set_attribute(STRIKE, value);
    }

    /// Sets the underline color.
    pub fn set_underline_color(&mut self, color: Option<Color>) {
        self.underline_color = color;
    }

    /// Sets the underline style.
    pub fn set_underline_style(&mut self, style: Option<UnderlineStyle>) {
        self.underline_style = style;
    }

    /// Combines multiple styles into one (left-to-right merge).
    ///
    /// Prefer [`Style::combine_refs`] in hot paths — `combine` clones each
    /// input on the way through `Add<Style>`. The reference-based variant
    /// avoids the per-step clone.
    pub fn combine(styles: &[Style]) -> Style {
        styles
            .iter()
            .fold(Style::null(), |acc, style| acc + style.clone())
    }

    /// Alias for [`Style::combine`] matching Python rich's `Style.chain`.
    ///
    /// Applies each style left-to-right; later styles override earlier ones
    /// for conflicting attributes.
    pub fn chain(styles: &[Style]) -> Style {
        Self::combine(styles)
    }

    /// Returns the canonical string form of this style.
    ///
    /// Equivalent to formatting and re-parsing: `Style::parse(&format!("{}", self))`.
    /// Useful for normalising user-supplied style strings (config, CLI flags)
    /// to a canonical representation.
    ///
    /// Mirrors Python rich's `Style.normalize(definition)`.
    pub fn normalize(&self) -> String {
        self.to_string()
    }

    /// Like [`Style::combine`] but iterates over references — avoids the
    /// per-step `Style.clone()` that the by-value `Add<Style>` impl forces.
    ///
    /// Used by the per-segment span-rendering loop in `Text` where the same
    /// active span set is consulted thousands of times per render.
    pub fn combine_refs<'a, I>(styles: I) -> Style
    where
        I: IntoIterator<Item = &'a Style>,
    {
        let mut acc = Style::null();
        for style in styles {
            // Manually inline Add::add with rhs taken by reference — clones
            // only the chosen Option fields, not the entire rhs Style.
            acc = Style {
                color: style.color.or(acc.color),
                bgcolor: style.bgcolor.or(acc.bgcolor),
                set_attributes: acc.set_attributes | style.set_attributes,
                attributes: (acc.attributes & !style.set_attributes)
                    | (style.attributes & style.set_attributes),
                link: style.link.clone().or(acc.link),
                underline_color: style.underline_color.or(acc.underline_color),
                underline_style: style.underline_style.or(acc.underline_style),
            };
        }
        acc
    }

    /// Renders text with this style as ANSI escape sequences.
    /// Render `text` with this style's SGR codes (color, bold, etc.) but
    /// **without** wrapping in an OSC 8 hyperlink even if `self.link` is set.
    ///
    /// Used by `Console::render_buffer` when it has decided to coalesce a
    /// run of consecutive same-link segments under a single OSC 8 wrapper.
    ///
    /// Returns plain `text` when `color_system` is `None` or `text` is empty.
    pub fn render_no_link(&self, text: &str, color_system: Option<ColorSystem>) -> String {
        // Skip the OSC 8 wrapping branch — no clone of `self` needed.
        self.render_inner(text, color_system, false)
    }

    pub fn render(&self, text: &str, color_system: Option<ColorSystem>) -> String {
        self.render_inner(text, color_system, true)
    }

    /// Internal render path. `emit_link` controls whether `self.link` is
    /// wrapped in OSC 8 (the public `render` passes `true`; `render_no_link`
    /// passes `false` to avoid cloning Style just to strip the field).
    fn render_inner(
        &self,
        text: &str,
        color_system: Option<ColorSystem>,
        emit_link: bool,
    ) -> String {
        if text.is_empty() || color_system.is_none() {
            return text.to_string();
        }

        // Build semicolon-separated SGR codes directly into a buffer.
        // Pre-size for the typical "bold + 256-color fg + 256-color bg"
        // sequence (≈ "1;38;5;NNN;48;5;NNN" → ~20 chars).
        let mut sgr = String::with_capacity(32);

        // Add attribute codes
        let attrs: [(u16, &str); 13] = [
            (BOLD, "1"),
            (DIM, "2"),
            (ITALIC, "3"),
            (UNDERLINE, "4"),
            (BLINK, "5"),
            (BLINK2, "6"),
            (REVERSE, "7"),
            (CONCEAL, "8"),
            (STRIKE, "9"),
            (UNDERLINE2, "21"),
            (FRAME, "51"),
            (ENCIRCLE, "52"),
            (OVERLINE, "53"),
        ];

        for (bit, code) in &attrs {
            if self.attributes & bit != 0 && self.set_attributes & bit != 0 {
                if !sgr.is_empty() {
                    sgr.push(';');
                }
                sgr.push_str(code);
            }
        }

        // Underline style codes (extended underline)
        if let Some(ul_style) = &self.underline_style {
            if !sgr.is_empty() {
                sgr.push(';');
            }
            sgr.push_str(match ul_style {
                UnderlineStyle::Single => "4:1",
                UnderlineStyle::Double => "4:2",
                UnderlineStyle::Curly => "4:3",
                UnderlineStyle::Dotted => "4:4",
                UnderlineStyle::Dashed => "4:5",
            });
        }

        // Add color codes
        if let Some(color) = &self.color {
            color.write_ansi_codes(true, &mut sgr);
        }

        if let Some(bgcolor) = &self.bgcolor {
            bgcolor.write_ansi_codes(false, &mut sgr);
        }

        // Underline color (SGR 58;5;N or 58;2;R;G;B)
        if let Some(ul_color) = &self.underline_color {
            ul_color.write_underline_color_codes(&mut sgr);
        }

        // Pre-size: text + SGR opener (~5 + sgr) + reset (~4) + slack.
        let mut result = String::with_capacity(text.len() + sgr.len() + 12);

        if sgr.is_empty() {
            result.push_str(text);
        } else {
            write!(result, "\x1b[{}m{}\x1b[0m", sgr, text).unwrap();
        }

        // Wrap in hyperlink if present and the caller wants links emitted.
        //
        // Emit `id=N;url` so terminals that group multi-line links (iTerm2,
        // Kitty, WezTerm) recognise runs as a single clickable target.
        // The id is derived deterministically from the URL string via a
        // FNV-1a 64-bit hash, so repeated `render()` calls on the same
        // Style (or different Style instances with the same link) produce
        // the same id — no mutable state or extra fields required.
        if emit_link {
            if let Some(url) = &self.link {
                let id = link_id_for(url);
                let mut linked = String::with_capacity(result.len() + url.len() + 32);
                write!(
                    linked,
                    "\x1b]8;id={};{}\x1b\\{}\x1b]8;;\x1b\\",
                    id, url, result
                )
                .unwrap();
                return linked;
            }
        }
        result
    }

    /// Pick the first non-null style from the candidate list, or [`Style::null`]
    /// if every candidate is null/None.
    ///
    /// Mirrors rich's `Style.pick_first(*candidates)` — useful in render
    /// pipelines that select among theme / row / column / cell style overrides.
    pub fn pick_first(candidates: &[Option<&Style>]) -> Style {
        for s in candidates.iter().flatten() {
            if !s.is_null() {
                return (*s).clone();
            }
        }
        Style::null()
    }

    /// Returns true if this is a null style (nothing set).
    pub fn is_null(&self) -> bool {
        self.color.is_none()
            && self.bgcolor.is_none()
            && self.set_attributes == 0
            && self.link.is_none()
            && self.underline_color.is_none()
            && self.underline_style.is_none()
    }

    // -- Typed builder methods (Task 3) ------------------------------------

    /// Set the foreground color from a typed [`Color`] value (builder-style).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::style::Style;
    /// use gilt::color::Color;
    ///
    /// let s = Style::null().fg(Color::from_rgb(255, 0, 0));
    /// assert!(s.color().is_some());
    /// ```
    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the background color from a typed [`Color`] value (builder-style).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::style::Style;
    /// use gilt::color::Color;
    ///
    /// let s = Style::null().bg(Color::from_rgb(0, 0, 255));
    /// assert!(s.bgcolor().is_some());
    /// ```
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bgcolor = Some(color);
        self
    }

    /// Set the underline color from a typed [`Color`] value (builder-style).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::style::Style;
    /// use gilt::color::Color;
    ///
    /// let s = Style::null().with_underline_color(Color::from_rgb(0, 255, 0));
    /// assert!(s.underline_color().is_some());
    /// ```
    #[must_use]
    pub fn with_underline_color(mut self, color: Color) -> Self {
        self.underline_color = Some(color);
        self
    }

    /// Returns a copy of this style without colors.
    pub fn without_color(&self) -> Style {
        Style {
            color: None,
            bgcolor: None,
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: self.link.clone(),
            underline_color: self.underline_color,
            underline_style: self.underline_style,
        }
    }

    /// Returns a style with only the background color.
    pub fn background_style(&self) -> Style {
        Style {
            color: None,
            bgcolor: self.bgcolor,
            set_attributes: 0,
            attributes: 0,
            link: None,
            underline_color: None,
            underline_style: None,
        }
    }

    /// Returns a copy without metadata and links.
    pub fn clear_meta_and_links(&self) -> Style {
        Style {
            color: self.color,
            bgcolor: self.bgcolor,
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: None,
            underline_color: self.underline_color,
            underline_style: self.underline_style,
        }
    }

    /// Returns a copy of this style with the given hyperlink URL.
    pub fn with_link(url: &str) -> Style {
        Style {
            color: None,
            bgcolor: None,
            set_attributes: 0,
            attributes: 0,
            link: Some(url.to_string()),
            underline_color: None,
            underline_style: None,
        }
    }

    /// Returns a copy with an updated link.
    pub fn update_link(&self, link: Option<&str>) -> Style {
        Style {
            color: self.color,
            bgcolor: self.bgcolor,
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: link.map(|s| s.to_string()),
            underline_color: self.underline_color,
            underline_style: self.underline_style,
        }
    }

    /// Returns a CSS style string for HTML rendering.
    pub fn get_html_style(&self, theme: Option<&TerminalTheme>) -> String {
        let mut css = String::new();

        let mut fg_color = self.color.as_ref();
        let mut bg_color = self.bgcolor.as_ref();

        // Handle reverse
        if self.reverse() == Some(true) {
            std::mem::swap(&mut fg_color, &mut bg_color);
        }

        // Get color triplets
        let mut fg_triplet = fg_color.map(|c| c.get_truecolor(theme, true));
        let bg_triplet = bg_color.map(|c| c.get_truecolor(theme, false));

        // Handle dim
        if self.dim() == Some(true) {
            if let (Some(fg), Some(bg)) = (fg_triplet, bg_triplet) {
                fg_triplet = Some(blend_rgb(fg, bg, 0.5));
            }
        }

        // Color
        if let Some(triplet) = fg_triplet {
            let hex = triplet.hex();
            write!(css, "color: {}; text-decoration-color: {}", hex, hex).unwrap();
        }

        // Background color
        if let Some(triplet) = bg_triplet {
            if !css.is_empty() {
                css.push_str("; ");
            }
            write!(css, "background-color: {}", triplet.hex()).unwrap();
        }

        // Bold
        if self.bold() == Some(true) {
            if !css.is_empty() {
                css.push_str("; ");
            }
            css.push_str("font-weight: bold");
        }

        // Italic
        if self.italic() == Some(true) {
            if !css.is_empty() {
                css.push_str("; ");
            }
            css.push_str("font-style: italic");
        }

        // Text decorations
        let has_underline = self.underline() == Some(true);
        let has_strike = self.strike() == Some(true);
        let has_overline = self.overline() == Some(true);
        if has_underline || has_strike || has_overline {
            if !css.is_empty() {
                css.push_str("; ");
            }
            css.push_str("text-decoration: ");
            let mut first = true;
            if has_underline {
                css.push_str("underline");
                first = false;
            }
            if has_strike {
                if !first {
                    css.push(' ');
                }
                css.push_str("line-through");
                first = false;
            }
            if has_overline {
                if !first {
                    css.push(' ');
                }
                css.push_str("overline");
            }
        }

        css
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        // Attributes
        let attrs = [
            (BOLD, "bold", "not bold"),
            (DIM, "dim", "not dim"),
            (ITALIC, "italic", "not italic"),
            (UNDERLINE, "underline", "not underline"),
            (BLINK, "blink", "not blink"),
            (BLINK2, "blink2", "not blink2"),
            (REVERSE, "reverse", "not reverse"),
            (CONCEAL, "conceal", "not conceal"),
            (STRIKE, "strike", "not strike"),
            (UNDERLINE2, "underline2", "not underline2"),
            (FRAME, "frame", "not frame"),
            (ENCIRCLE, "encircle", "not encircle"),
            (OVERLINE, "overline", "not overline"),
        ];

        for (bit, on_name, off_name) in &attrs {
            if self.set_attributes & bit != 0 {
                if self.attributes & bit != 0 {
                    parts.push(on_name.to_string());
                } else {
                    parts.push(off_name.to_string());
                }
            }
        }

        // Foreground color
        if let Some(color) = &self.color {
            parts.push(color.name().into_owned());
        }

        // Background color
        if let Some(bgcolor) = &self.bgcolor {
            parts.push("on".to_string());
            parts.push(bgcolor.name().into_owned());
        }

        // Underline style — match to a static &'static str (avoids two
        // allocations from `format!("{:?}").to_lowercase()`).
        if let Some(ul_style) = &self.underline_style {
            parts.push(
                match ul_style {
                    UnderlineStyle::Single => "single",
                    UnderlineStyle::Double => "double",
                    UnderlineStyle::Curly => "curly",
                    UnderlineStyle::Dotted => "dotted",
                    UnderlineStyle::Dashed => "dashed",
                }
                .to_string(),
            );
        }

        // Underline color
        if let Some(ul_color) = &self.underline_color {
            parts.push(format!("underline_color({})", ul_color.name()));
        }

        // Link
        if let Some(link) = &self.link {
            parts.push("link".to_string());
            parts.push(link.clone());
        }

        if parts.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

impl PartialEq for Style {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.bgcolor == other.bgcolor
            && self.set_attributes == other.set_attributes
            && self.attributes == other.attributes
            && self.link == other.link
            && self.underline_color == other.underline_color
            && self.underline_style == other.underline_style
    }
}

impl std::hash::Hash for Style {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.bgcolor.hash(state);
        self.set_attributes.hash(state);
        self.attributes.hash(state);
        self.link.hash(state);
        self.underline_color.hash(state);
        self.underline_style.hash(state);
    }
}

impl Eq for Style {}

impl Add<Style> for Style {
    type Output = Style;

    fn add(self, rhs: Style) -> Style {
        Style {
            color: rhs.color.or(self.color),
            bgcolor: rhs.bgcolor.or(self.bgcolor),
            set_attributes: self.set_attributes | rhs.set_attributes,
            attributes: (self.attributes & !rhs.set_attributes)
                | (rhs.attributes & rhs.set_attributes),
            link: rhs.link.or(self.link),
            underline_color: rhs.underline_color.or(self.underline_color),
            underline_style: rhs.underline_style.or(self.underline_style),
        }
    }
}

impl Add<Option<Style>> for Style {
    type Output = Style;

    fn add(self, rhs: Option<Style>) -> Style {
        match rhs {
            Some(style) => self + style,
            None => self,
        }
    }
}

/// Parses an underline-style token (emitted by [`Display`]) to its variant.
fn parse_underline_style_name(name: &str) -> Option<UnderlineStyle> {
    match name {
        "single" => Some(UnderlineStyle::Single),
        "double" => Some(UnderlineStyle::Double),
        "curly" => Some(UnderlineStyle::Curly),
        "dotted" => Some(UnderlineStyle::Dotted),
        "dashed" => Some(UnderlineStyle::Dashed),
        _ => None,
    }
}

/// Parses an attribute name to its bit mask.
fn parse_attribute_name(name: &str) -> Option<u16> {
    match name {
        "bold" | "b" => Some(BOLD),
        "dim" | "d" => Some(DIM),
        "italic" | "i" => Some(ITALIC),
        "underline" | "u" => Some(UNDERLINE),
        "blink" => Some(BLINK),
        "blink2" => Some(BLINK2),
        "reverse" | "r" => Some(REVERSE),
        "conceal" | "c" => Some(CONCEAL),
        "strike" | "s" => Some(STRIKE),
        "underline2" | "uu" => Some(UNDERLINE2),
        "frame" => Some(FRAME),
        "encircle" => Some(ENCIRCLE),
        "overline" | "o" => Some(OVERLINE),
        _ => None,
    }
}

/// A stack of styles for managing nested style contexts.
#[derive(Debug, Clone)]
pub struct StyleStack {
    stack: Vec<Style>,
}

impl StyleStack {
    /// Creates a new style stack with a default style.
    pub fn new(default: Style) -> Self {
        StyleStack {
            stack: vec![default],
        }
    }

    /// Returns the current (top) style.
    pub fn current(&self) -> &Style {
        self.stack.last().expect("StyleStack should never be empty")
    }

    /// Pushes a new style, combining it with the current style.
    pub fn push(&mut self, style: Style) {
        let new_style = self.current().clone() + style;
        self.stack.push(new_style);
    }

    /// Pops the top style and returns the new current style.
    pub fn pop(&mut self) -> Result<&Style, StyleError> {
        if self.stack.len() <= 1 {
            return Err(StyleError::StackError(
                "cannot pop from stack with only default style".to_string(),
            ));
        }
        self.stack.pop();
        Ok(self.current())
    }
}

// ============================================================================
// LRU Cache for Style Parsing
// ============================================================================

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Process-wide monotonic counter for OSC 8 `id=` parameters.
///
/// Retained for tests that assert monotonic behaviour; the render path now
/// uses [`link_id_for`] instead.
static LINK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Return a fresh OSC 8 link id, monotonically increasing for the process.
pub(crate) fn next_link_id() -> u64 {
    LINK_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Derive a stable OSC 8 `id=` value from a URL string using FNV-1a 64-bit.
///
/// Using a hash of the URL means identical links on different Style instances
/// produce the same id, and repeated `render()` calls on the same Style are
/// idempotent — no mutable state or extra fields needed.
fn link_id_for(url: &str) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut hash = FNV_OFFSET;
    for byte in url.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Global LRU cache for parsed styles with capacity for 256 entries.
static STYLE_CACHE: Mutex<Option<LruCache<String, Style>>> = Mutex::new(None);

/// Gets or initializes the style cache.
///
/// Recovers from a poisoned mutex (after a panic in a previous holder) by
/// extracting the inner value — the cache is purely a parse accelerator, so
/// the data behind a poison flag is still safe to use.
fn get_style_cache() -> std::sync::MutexGuard<'static, Option<LruCache<String, Style>>> {
    let mut cache = STYLE_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cache.is_none() {
        *cache = Some(LruCache::new(NonZeroUsize::new(256).unwrap()));
    }
    cache
}

/// Clears the global style cache.
pub fn clear_style_cache() {
    if let Ok(mut cache) = STYLE_CACHE.lock() {
        *cache = None;
    }
}

/// Returns the current number of entries in the style cache.
pub fn style_cache_size() -> usize {
    if let Ok(cache) = STYLE_CACHE.lock() {
        cache.as_ref().map(|c| c.len()).unwrap_or(0)
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// serde Serialize / Deserialize for Style (gated on `json` feature)
// ---------------------------------------------------------------------------
//
// `Style` serializes as its canonical `Display` string (e.g. `"bold red"`,
// `"on blue"`, `"none"`) and deserializes via `Style::parse_strict`.
// A custom impl is used because the bit-packed internal representation has
// no meaningful automatic serde mapping.

#[cfg(feature = "json")]
impl serde::Serialize for Style {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for Style {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{self, Visitor};
        struct StyleVisitor;
        impl<'de> Visitor<'de> for StyleVisitor {
            type Value = Style;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a style string like \"bold red\" or \"italic on blue\"")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Style, E> {
                Style::parse_strict(v).map_err(|e| de::Error::custom(e.to_string()))
            }
        }
        d.deserialize_str(StyleVisitor)
    }
}

#[cfg(test)]
#[path = "style_tests.rs"]
mod tests;
