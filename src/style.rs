//! Style representation and manipulation for terminal text.
//!
//! This module provides the Style type that represents the visual appearance
//! of terminal text, including colors, text attributes (bold, italic, etc.),
//! and hyperlinks.

use crate::color::{blend_rgb, Color, ColorSystem};
use crate::errors::StyleError;
use crate::terminal_theme::TerminalTheme;
use std::fmt;
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

    /// Parses a style definition string.
    ///
    /// # Grammar
    /// - Words are split by whitespace
    /// - "on" keyword: next word is background color
    /// - "not" keyword: next word is attribute name, set to false
    /// - "link" keyword: next word is URL
    /// - Known attribute names with aliases
    /// - Anything else: try as foreground color
    pub fn parse(definition: &str) -> Result<Self, StyleError> {
        let definition = definition.trim();
        if definition.is_empty() {
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
    pub fn combine(styles: &[Style]) -> Style {
        styles
            .iter()
            .fold(Style::null(), |acc, style| acc + style.clone())
    }

    /// Renders text with this style as ANSI escape sequences.
    pub fn render(&self, text: &str, color_system: Option<ColorSystem>) -> String {
        if text.is_empty() || color_system.is_none() {
            return text.to_string();
        }

        let mut codes = Vec::new();

        // Add attribute codes
        let attrs = [
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
                codes.push(code.to_string());
            }
        }

        // Underline style codes (extended underline)
        if let Some(ul_style) = &self.underline_style {
            match ul_style {
                UnderlineStyle::Single => codes.push("4:1".to_string()),
                UnderlineStyle::Double => codes.push("4:2".to_string()),
                UnderlineStyle::Curly => codes.push("4:3".to_string()),
                UnderlineStyle::Dotted => codes.push("4:4".to_string()),
                UnderlineStyle::Dashed => codes.push("4:5".to_string()),
            }
        }

        // Add color codes
        if let Some(color) = &self.color {
            codes.extend(color.get_ansi_codes(true));
        }

        if let Some(bgcolor) = &self.bgcolor {
            codes.extend(bgcolor.get_ansi_codes(false));
        }

        // Underline color (SGR 58;5;N or 58;2;R;G;B)
        if let Some(ul_color) = &self.underline_color {
            let ul_codes = ul_color.get_ansi_codes(true);
            // Convert foreground codes to underline color codes (38->58)
            if !ul_codes.is_empty() {
                let first = &ul_codes[0];
                if first == "38" {
                    codes.push("58".to_string());
                    codes.extend(ul_codes[1..].iter().cloned());
                } else {
                    // Standard color: convert 3x to 58;5;N
                    // Standard colors use codes 30-37, map to 0-7 for 58;5;N
                    if let Ok(code_num) = first.parse::<u8>() {
                        if (30..=37).contains(&code_num) {
                            codes.push("58".to_string());
                            codes.push("5".to_string());
                            codes.push(format!("{}", code_num - 30));
                        } else if (90..=97).contains(&code_num) {
                            codes.push("58".to_string());
                            codes.push("5".to_string());
                            codes.push(format!("{}", code_num - 90 + 8));
                        }
                    }
                }
            }
        }

        let rendered = if codes.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
        };

        // Wrap in hyperlink if present
        if let Some(url) = &self.link {
            format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, rendered)
        } else {
            rendered
        }
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

    /// Returns a copy of this style without colors.
    pub fn without_color(&self) -> Style {
        Style {
            color: None,
            bgcolor: None,
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: self.link.clone(),
            underline_color: self.underline_color.clone(),
            underline_style: self.underline_style,
        }
    }

    /// Returns a style with only the background color.
    pub fn background_style(&self) -> Style {
        Style {
            color: None,
            bgcolor: self.bgcolor.clone(),
            set_attributes: 0,
            attributes: 0,
            link: None,
            underline_color: None,
            underline_style: None,
        }
    }

    /// Returns a deep copy of this style.
    pub fn copy(&self) -> Style {
        self.clone()
    }

    /// Returns a copy without metadata and links.
    pub fn clear_meta_and_links(&self) -> Style {
        Style {
            color: self.color.clone(),
            bgcolor: self.bgcolor.clone(),
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: None,
            underline_color: self.underline_color.clone(),
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
            color: self.color.clone(),
            bgcolor: self.bgcolor.clone(),
            set_attributes: self.set_attributes,
            attributes: self.attributes,
            link: link.map(|s| s.to_string()),
            underline_color: self.underline_color.clone(),
            underline_style: self.underline_style,
        }
    }

    /// Returns a CSS style string for HTML rendering.
    pub fn get_html_style(&self, theme: Option<&TerminalTheme>) -> String {
        let mut styles = Vec::new();

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
            styles.push(format!("color: {}", triplet.hex()));
            styles.push(format!("text-decoration-color: {}", triplet.hex()));
        }

        // Background color
        if let Some(triplet) = bg_triplet {
            styles.push(format!("background-color: {}", triplet.hex()));
        }

        // Bold
        if self.bold() == Some(true) {
            styles.push("font-weight: bold".to_string());
        }

        // Italic
        if self.italic() == Some(true) {
            styles.push("font-style: italic".to_string());
        }

        // Text decorations
        let mut decorations = Vec::new();
        if self.underline() == Some(true) {
            decorations.push("underline");
        }
        if self.strike() == Some(true) {
            decorations.push("line-through");
        }
        if self.overline() == Some(true) {
            decorations.push("overline");
        }
        if !decorations.is_empty() {
            styles.push(format!("text-decoration: {}", decorations.join(" ")));
        }

        styles.join("; ")
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
            parts.push(color.name.clone());
        }

        // Background color
        if let Some(bgcolor) = &self.bgcolor {
            parts.push("on".to_string());
            parts.push(bgcolor.name.clone());
        }

        // Underline style
        if let Some(ul_style) = &self.underline_style {
            parts.push(format!("{:?}", ul_style).to_lowercase());
        }

        // Underline color
        if let Some(ul_color) = &self.underline_color {
            parts.push(format!("underline_color({})", ul_color.name));
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

#[cfg(test)]
mod tests {
    use super::*;

    // Display tests
    #[test]
    fn test_display_not_bold() {
        let style = Style::new(
            None,
            None,
            Some(false),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(style.to_string(), "not bold");
    }

    #[test]
    fn test_display_not_bold_with_color() {
        let style = Style::new(
            Some("red"),
            None,
            Some(false),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(style.to_string(), "not bold red");
    }

    #[test]
    fn test_display_null() {
        let style = Style::null();
        assert_eq!(style.to_string(), "none");
    }

    #[test]
    fn test_display_bold() {
        let style = Style::new(
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(style.to_string(), "bold");
    }

    #[test]
    fn test_display_bold_red_on_black() {
        let style = Style::new(
            Some("red"),
            Some("black"),
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(style.to_string(), "bold red on black");
    }

    #[test]
    fn test_display_link() {
        let style = Style::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("foo"),
        )
        .unwrap();
        assert_eq!(style.to_string(), "link foo");
    }

    #[test]
    fn test_display_all_attributes() {
        let style = Style::new(
            Some("red"),
            Some("black"),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            None,
        )
        .unwrap();
        let s = style.to_string();
        assert!(s.contains("bold"));
        assert!(s.contains("dim"));
        assert!(s.contains("italic"));
        assert!(s.contains("underline"));
        assert!(s.contains("blink"));
        assert!(s.contains("blink2"));
        assert!(s.contains("reverse"));
        assert!(s.contains("conceal"));
        assert!(s.contains("strike"));
        assert!(s.contains("underline2"));
        assert!(s.contains("frame"));
        assert!(s.contains("encircle"));
        assert!(s.contains("overline"));
        assert!(s.contains("red"));
        assert!(s.contains("on black"));
    }

    // Equality tests
    #[test]
    fn test_equality_same() {
        let style1 = Style::new(
            Some("red"),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let style2 = Style::new(
            Some("red"),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(style1, style2);
    }

    #[test]
    fn test_equality_different_color() {
        let style1 = Style::new(
            Some("red"),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let style2 = Style::new(
            Some("green"),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_ne!(style1, style2);
    }

    // is_null tests
    #[test]
    fn test_is_null_true() {
        let style = Style::null();
        assert!(style.is_null());
    }

    #[test]
    fn test_is_null_false_with_bold() {
        let style = Style::new(
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(!style.is_null());
    }

    #[test]
    fn test_is_null_false_with_color() {
        let style = Style::new(
            Some("red"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(!style.is_null());
    }

    // Parse tests
    #[test]
    fn test_parse_empty() {
        let style = Style::parse("").unwrap();
        assert!(style.is_null());
    }

    #[test]
    fn test_parse_red() {
        let style = Style::parse("red").unwrap();
        assert_eq!(style.color().unwrap().name, "red");
    }

    #[test]
    fn test_parse_not_bold() {
        let style = Style::parse("not bold").unwrap();
        assert_eq!(style.bold(), Some(false));
    }

    #[test]
    fn test_parse_bold_red_on_black() {
        let style = Style::parse("bold red on black").unwrap();
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.color().unwrap().name, "red");
        assert_eq!(style.bgcolor().unwrap().name, "black");
    }

    #[test]
    fn test_parse_bold_link() {
        let style = Style::parse("bold link https://example.org").unwrap();
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.link(), Some("https://example.org"));
    }

    #[test]
    fn test_parse_error_on_alone() {
        let result = Style::parse("on");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_on_invalid_color() {
        let result = Style::parse("on nothing");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_rgb_out_of_range() {
        let result = Style::parse("rgb(999,999,999)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_not_unknown_attribute() {
        let result = Style::parse("not monkey");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_link_alone() {
        let result = Style::parse("link");
        assert!(result.is_err());
    }

    // Render tests
    #[test]
    fn test_render_no_color_system() {
        let style = Style::parse("red").unwrap();
        assert_eq!(style.render("foo", None), "foo");
    }

    #[test]
    fn test_render_empty_text() {
        let style = Style::parse("red").unwrap();
        assert_eq!(style.render("", Some(ColorSystem::TrueColor)), "");
    }

    #[test]
    fn test_render_null_style() {
        let style = Style::null();
        assert_eq!(style.render("foo", Some(ColorSystem::TrueColor)), "foo");
    }

    #[test]
    fn test_render_bold_red_on_black() {
        let style = Style::parse("bold red on black").unwrap();
        let rendered = style.render("foo", Some(ColorSystem::TrueColor));
        assert!(rendered.contains("\x1b[1;31;40m"));
        assert!(rendered.contains("foo"));
        assert!(rendered.contains("\x1b[0m"));
    }

    #[test]
    fn test_render_all_attributes() {
        let style = Style::parse(
            "bold dim italic underline blink blink2 reverse conceal strike underline2 frame encircle overline red on black"
        ).unwrap();
        let rendered = style.render("foo", Some(ColorSystem::TrueColor));
        assert!(rendered.contains("1;2;3;4;5;6;7;8;9;21;51;52;53;31;40"));
    }

    // Add tests
    #[test]
    fn test_add_with_none() {
        let style = Style::parse("red").unwrap();
        let result = style.clone() + None;
        assert_eq!(result, style);
    }

    #[test]
    fn test_add_styles() {
        let style1 = Style::parse("red").unwrap();
        let style2 = Style::parse("bold").unwrap();
        let result = style1 + style2;
        assert_eq!(result.color().unwrap().name, "red");
        assert_eq!(result.bold(), Some(true));
    }

    #[test]
    fn test_add_override_color() {
        let style1 = Style::parse("red").unwrap();
        let style2 = Style::parse("blue").unwrap();
        let result = style1 + style2;
        assert_eq!(result.color().unwrap().name, "blue");
    }

    // StyleStack tests
    #[test]
    fn test_style_stack_new() {
        let stack = StyleStack::new(Style::parse("red").unwrap());
        assert_eq!(stack.current().color().unwrap().name, "red");
    }

    #[test]
    fn test_style_stack_push() {
        let mut stack = StyleStack::new(Style::parse("red").unwrap());
        stack.push(Style::parse("bold").unwrap());
        assert_eq!(stack.current().color().unwrap().name, "red");
        assert_eq!(stack.current().bold(), Some(true));
    }

    #[test]
    fn test_style_stack_pop() {
        let mut stack = StyleStack::new(Style::parse("red").unwrap());
        stack.push(Style::parse("bold").unwrap());
        stack.pop().unwrap();
        assert_eq!(stack.current().color().unwrap().name, "red");
        assert_eq!(stack.current().bold(), None);
    }

    #[test]
    fn test_style_stack_pop_error() {
        let mut stack = StyleStack::new(Style::null());
        let result = stack.pop();
        assert!(result.is_err());
    }

    // HTML style tests
    #[test]
    fn test_get_html_style_complex() {
        let style =
            Style::parse("reverse dim red on blue bold italic underline strike overline").unwrap();
        let html = style.get_html_style(None);
        // With reverse: blue becomes fg, red becomes bg
        // With dim: blend blue (0,0,128) with red (128,0,0) at 50% = (64,0,64) = #400040
        assert!(html.contains("color: #400040"));
        assert!(html.contains("text-decoration-color: #400040"));
        assert!(html.contains("background-color: #800000"));
        assert!(html.contains("font-weight: bold"));
        assert!(html.contains("font-style: italic"));
        assert!(html.contains("text-decoration: underline line-through overline"));
    }

    #[test]
    fn test_get_html_style_simple() {
        let style = Style::parse("bold red").unwrap();
        let html = style.get_html_style(None);
        assert!(html.contains("color: #800000"));
        assert!(html.contains("font-weight: bold"));
    }

    // without_color tests
    #[test]
    fn test_without_color() {
        let style = Style::parse("bold red on blue").unwrap();
        let without = style.without_color();
        assert_eq!(without.bold(), Some(true));
        assert!(without.color().is_none());
        assert!(without.bgcolor().is_none());
    }

    // background_style tests
    #[test]
    fn test_background_style() {
        let style = Style::parse("bold yellow on red").unwrap();
        let bg = style.background_style();
        assert!(bg.color().is_none());
        assert_eq!(bg.bgcolor().unwrap().name, "red");
        assert_eq!(bg.bold(), None);
    }

    // clear_meta_and_links tests
    #[test]
    fn test_clear_meta_and_links() {
        let style = Style::parse("bold red link https://example.org").unwrap();
        let cleared = style.clear_meta_and_links();
        assert_eq!(cleared.bold(), Some(true));
        assert_eq!(cleared.color().unwrap().name, "red");
        assert!(cleared.link().is_none());
    }

    // Combine tests
    #[test]
    fn test_combine_empty() {
        let result = Style::combine(&[]);
        assert!(result.is_null());
    }

    #[test]
    fn test_combine_multiple() {
        let styles = vec![
            Style::parse("red").unwrap(),
            Style::parse("bold").unwrap(),
            Style::parse("on blue").unwrap(),
        ];
        let result = Style::combine(&styles);
        assert_eq!(result.color().unwrap().name, "red");
        assert_eq!(result.bold(), Some(true));
        assert_eq!(result.bgcolor().unwrap().name, "blue");
    }

    // Attribute aliases tests
    #[test]
    fn test_parse_attribute_alias_b() {
        let style = Style::parse("b").unwrap();
        assert_eq!(style.bold(), Some(true));
    }

    #[test]
    fn test_parse_attribute_alias_i() {
        let style = Style::parse("i").unwrap();
        assert_eq!(style.italic(), Some(true));
    }

    #[test]
    fn test_parse_attribute_alias_u() {
        let style = Style::parse("u").unwrap();
        assert_eq!(style.underline(), Some(true));
    }

    #[test]
    fn test_parse_attribute_alias_s() {
        let style = Style::parse("s").unwrap();
        assert_eq!(style.strike(), Some(true));
    }

    #[test]
    fn test_parse_attribute_alias_uu() {
        let style = Style::parse("uu").unwrap();
        assert_eq!(style.underline2(), Some(true));
    }

    #[test]
    fn test_parse_attribute_alias_o() {
        let style = Style::parse("o").unwrap();
        assert_eq!(style.overline(), Some(true));
    }

    // Hash test
    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let style1 = Style::parse("bold red").unwrap();
        let style2 = Style::parse("bold red").unwrap();
        let style3 = Style::parse("bold blue").unwrap();

        set.insert(style1);
        assert!(set.contains(&style2));
        set.insert(style3);
        assert_eq!(set.len(), 2);
    }

    // Copy test
    #[test]
    fn test_copy() {
        let style = Style::parse("bold red").unwrap();
        let copied = style.copy();
        assert_eq!(style, copied);
    }

    // from_color tests
    #[test]
    fn test_from_color() {
        let color = Color::parse("red").unwrap();
        let bgcolor = Color::parse("blue").unwrap();
        let style = Style::from_color(Some(color), Some(bgcolor));
        assert_eq!(style.color().unwrap().name, "red");
        assert_eq!(style.bgcolor().unwrap().name, "blue");
        assert!(style.bold().is_none());
    }

    #[test]
    fn test_from_color_none() {
        let style = Style::from_color(None, None);
        assert!(style.color().is_none());
        assert!(style.bgcolor().is_none());
    }

    // -- with_link builder tests --------------------------------------------

    #[test]
    fn test_with_link() {
        let style = Style::with_link("https://example.com");
        assert_eq!(style.link(), Some("https://example.com"));
        assert!(style.color().is_none());
        assert!(style.bold().is_none());
    }

    #[test]
    fn test_with_link_is_not_null() {
        let style = Style::with_link("https://example.com");
        assert!(!style.is_null());
    }

    // -- link=URL parse syntax tests ----------------------------------------

    #[test]
    fn test_parse_link_equals_syntax() {
        let style = Style::parse("link=https://example.com").unwrap();
        assert_eq!(style.link(), Some("https://example.com"));
    }

    #[test]
    fn test_parse_bold_link_equals_syntax() {
        let style = Style::parse("bold link=https://example.com").unwrap();
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.link(), Some("https://example.com"));
    }

    #[test]
    fn test_parse_link_equals_empty_error() {
        let result = Style::parse("link=");
        assert!(result.is_err());
    }

    // -- link rendering tests -----------------------------------------------

    #[test]
    fn test_render_link_only() {
        let style = Style::with_link("https://example.com");
        let rendered = style.render("click", Some(ColorSystem::TrueColor));
        assert_eq!(
            rendered,
            "\x1b]8;;https://example.com\x1b\\click\x1b]8;;\x1b\\"
        );
    }

    #[test]
    fn test_render_bold_with_link() {
        let style = Style::parse("bold link https://example.com").unwrap();
        let rendered = style.render("click", Some(ColorSystem::TrueColor));
        // Should have OSC 8 wrapping around the ANSI-styled text
        assert!(rendered.starts_with("\x1b]8;;https://example.com\x1b\\"));
        assert!(rendered.ends_with("\x1b]8;;\x1b\\"));
        assert!(rendered.contains("\x1b[1m"));
        assert!(rendered.contains("click"));
    }

    #[test]
    fn test_render_link_no_color_system() {
        // With no color system, render returns plain text (no link wrapping)
        let style = Style::with_link("https://example.com");
        let rendered = style.render("click", None);
        assert_eq!(rendered, "click");
    }

    // -- link combine tests -------------------------------------------------

    #[test]
    fn test_add_link_override() {
        let style1 = Style::parse("link https://a.com").unwrap();
        let style2 = Style::parse("link https://b.com").unwrap();
        let result = style1 + style2;
        assert_eq!(result.link(), Some("https://b.com"));
    }

    #[test]
    fn test_add_link_preserved() {
        let style1 = Style::parse("link https://a.com").unwrap();
        let style2 = Style::parse("bold").unwrap();
        let result = style1 + style2;
        assert_eq!(result.link(), Some("https://a.com"));
        assert_eq!(result.bold(), Some(true));
    }

    #[test]
    fn test_combine_link() {
        let styles = vec![
            Style::parse("red").unwrap(),
            Style::with_link("https://example.com"),
            Style::parse("bold").unwrap(),
        ];
        let result = Style::combine(&styles);
        assert_eq!(result.link(), Some("https://example.com"));
        assert_eq!(result.color().unwrap().name, "red");
        assert_eq!(result.bold(), Some(true));
    }

    // -- Underline enhancement tests ----------------------------------------

    #[test]
    fn test_underline_style_setter_getter() {
        let mut style = Style::null();
        assert!(style.underline_style().is_none());
        style.set_underline_style(Some(UnderlineStyle::Curly));
        assert_eq!(style.underline_style(), Some(UnderlineStyle::Curly));
    }

    #[test]
    fn test_underline_color_setter_getter() {
        let mut style = Style::null();
        assert!(style.underline_color().is_none());
        let red = Color::parse("red").unwrap();
        style.set_underline_color(Some(red));
        assert!(style.underline_color().is_some());
        assert_eq!(style.underline_color().unwrap().name, "red");
    }

    #[test]
    fn test_underline_style_is_not_null() {
        let mut style = Style::null();
        style.set_underline_style(Some(UnderlineStyle::Double));
        assert!(!style.is_null());
    }

    #[test]
    fn test_underline_color_is_not_null() {
        let mut style = Style::null();
        style.set_underline_color(Some(Color::parse("red").unwrap()));
        assert!(!style.is_null());
    }

    #[test]
    fn test_underline_style_display() {
        let mut style = Style::null();
        style.set_underline_style(Some(UnderlineStyle::Curly));
        assert!(style.to_string().contains("curly"));
    }

    #[test]
    fn test_underline_color_display() {
        let mut style = Style::null();
        style.set_underline_color(Some(Color::parse("red").unwrap()));
        assert!(style.to_string().contains("underline_color(red)"));
    }

    #[test]
    fn test_underline_style_add() {
        let mut s1 = Style::null();
        s1.set_underline_style(Some(UnderlineStyle::Curly));
        let mut s2 = Style::null();
        s2.set_underline_style(Some(UnderlineStyle::Dashed));
        let result = s1 + s2;
        assert_eq!(result.underline_style(), Some(UnderlineStyle::Dashed));
    }

    #[test]
    fn test_underline_color_add() {
        let mut s1 = Style::null();
        s1.set_underline_color(Some(Color::parse("red").unwrap()));
        let s2 = Style::parse("bold").unwrap();
        let result = s1 + s2;
        assert_eq!(result.underline_color().unwrap().name, "red");
    }

    #[test]
    fn test_underline_style_render_curly() {
        let mut style = Style::null();
        style.set_underline_style(Some(UnderlineStyle::Curly));
        let rendered = style.render("foo", Some(ColorSystem::TrueColor));
        assert!(rendered.contains("4:3"));
    }

    #[test]
    fn test_underline_style_render_dashed() {
        let mut style = Style::null();
        style.set_underline_style(Some(UnderlineStyle::Dashed));
        let rendered = style.render("foo", Some(ColorSystem::TrueColor));
        assert!(rendered.contains("4:5"));
    }

    #[test]
    fn test_underline_color_render_truecolor() {
        let mut style = Style::null();
        style.set_underline(Some(true));
        style.set_underline_color(Some(Color::from_rgb(255, 0, 0)));
        let rendered = style.render("foo", Some(ColorSystem::TrueColor));
        // Should contain 58;2;255;0;0 for underline color
        assert!(rendered.contains("58;2;255;0;0"), "rendered: {}", rendered);
    }

    #[test]
    fn test_without_color_preserves_underline_color() {
        let mut style = Style::parse("bold red on blue").unwrap();
        style.set_underline_color(Some(Color::parse("green").unwrap()));
        let without = style.without_color();
        assert!(without.color().is_none());
        assert!(without.bgcolor().is_none());
        assert!(without.underline_color().is_some());
        assert_eq!(without.underline_color().unwrap().name, "green");
    }

    #[test]
    fn test_background_style_clears_underline() {
        let mut style = Style::parse("bold red on blue").unwrap();
        style.set_underline_color(Some(Color::parse("green").unwrap()));
        style.set_underline_style(Some(UnderlineStyle::Curly));
        let bg = style.background_style();
        assert!(bg.underline_color().is_none());
        assert!(bg.underline_style().is_none());
    }

    #[test]
    fn test_underline_equality() {
        let mut s1 = Style::null();
        s1.set_underline_style(Some(UnderlineStyle::Curly));
        let mut s2 = Style::null();
        s2.set_underline_style(Some(UnderlineStyle::Curly));
        assert_eq!(s1, s2);

        let mut s3 = Style::null();
        s3.set_underline_style(Some(UnderlineStyle::Dashed));
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_public_setters() {
        let mut style = Style::null();
        style.set_bold(Some(true));
        style.set_dim(Some(true));
        style.set_italic(Some(true));
        style.set_underline(Some(true));
        style.set_blink(Some(true));
        style.set_reverse(Some(true));
        style.set_conceal(Some(true));
        style.set_strike(Some(true));

        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.dim(), Some(true));
        assert_eq!(style.italic(), Some(true));
        assert_eq!(style.underline(), Some(true));
        assert_eq!(style.blink(), Some(true));
        assert_eq!(style.reverse(), Some(true));
        assert_eq!(style.conceal(), Some(true));
        assert_eq!(style.strike(), Some(true));
    }
}
