//! Theme and ThemeStack for managing named style collections.
//!
//! A Theme maps style names to Style instances, optionally inheriting from
//! the default styles. ThemeStack manages a stack of themes for nested
//! style overrides (e.g., in console rendering).

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::Path;

use crate::default_styles::DEFAULT_STYLES;
use crate::error::StyleError;
use crate::style::Style;

/// A collection of named styles, optionally inheriting from defaults.
#[derive(Debug, Clone)]
pub struct Theme {
    /// The mapping from style names to Style instances.
    pub styles: HashMap<String, Style>,
}

impl Theme {
    /// Creates a new Theme.
    ///
    /// If `inherit` is true (the default behavior), the theme starts with all
    /// default styles and then overlays the provided styles on top.
    ///
    /// Style values in the `styles` map can be provided as Style instances.
    /// For string-based construction, callers should parse strings into Styles
    /// before passing them in.
    pub fn new(styles: Option<HashMap<String, Style>>, inherit: bool) -> Self {
        let mut merged = if inherit {
            DEFAULT_STYLES.clone()
        } else {
            HashMap::new()
        };

        if let Some(s) = styles {
            merged.extend(s);
        }

        Theme { styles: merged }
    }

    /// Looks up a style by name.
    pub fn get(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    /// Returns an INI-format config string representing this theme.
    ///
    /// The output is compatible with the Theme.config property:
    /// ```text
    /// [styles]
    /// bar.back = grey23
    /// bar.complete = rgb(249,38,114)
    /// ...
    /// ```
    pub fn config(&self) -> String {
        let mut entries: Vec<(&String, &Style)> = self.styles.iter().collect();
        entries.sort_by_key(|(name, _)| *name);

        let mut result = String::from("[styles]\n");
        for (name, style) in entries {
            result.push_str(&format!("{} = {}\n", name, style));
        }
        result
    }

    /// Alias for [`config`](Theme::config) — exports the theme as an INI-style string.
    pub fn to_config(&self) -> String {
        self.config()
    }

    /// Parses INI-style theme content into a Theme.
    ///
    /// Expected format:
    /// ```text
    /// [styles]
    /// info = dim cyan
    /// warning = magenta
    /// danger = bold red
    /// ```
    ///
    /// Blank lines and `#` comments are ignored. All style definitions
    /// are parsed via [`Style::parse`].
    ///
    /// The resulting theme does **not** inherit from defaults — it contains
    /// only the styles explicitly listed in the content.
    pub fn from_str(content: &str, inherit: bool) -> Result<Self, ThemeFromStrError> {
        let mut styles = HashMap::new();
        let mut in_styles_section = false;

        for (line_no, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();

            // Skip blank lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                let section = &line[1..line.len() - 1];
                in_styles_section = section.eq_ignore_ascii_case("styles");
                continue;
            }

            if !in_styles_section {
                continue;
            }

            // Parse "name = style_definition"
            if let Some(eq_pos) = line.find('=') {
                let name = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                if name.is_empty() {
                    return Err(ThemeFromStrError::Parse(format!(
                        "line {}: empty style name",
                        line_no + 1
                    )));
                }

                let style = Style::parse_strict(value).map_err(|e| ThemeFromStrError::Style {
                    name: name.to_string(),
                    source: e,
                })?;

                styles.insert(name.to_string(), style);
            } else {
                return Err(ThemeFromStrError::Parse(format!(
                    "line {}: expected 'name = style', got: {}",
                    line_no + 1,
                    line
                )));
            }
        }

        Ok(Theme::new(Some(styles), inherit))
    }

    /// Reads theme content from a file path.
    ///
    /// The file should contain INI-style theme content as described in
    /// [`Theme::from_str`]. If `inherit` is true, default styles are included;
    /// if false, only the styles in the file are loaded.
    pub fn from_file(path: &Path, inherit: bool) -> Result<Self, io::Error> {
        let content = std::fs::read_to_string(path)?;
        Theme::from_str(&content, inherit)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Reads theme content from any reader.
    ///
    /// The reader should provide INI-style theme content as described in
    /// [`Theme::from_str`]. If `inherit` is true, default styles are included;
    /// if false, only the styles from the reader are loaded.
    pub fn read(reader: &mut impl io::Read, inherit: bool) -> Result<Self, io::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Theme::from_str(&content, inherit)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// JSON helpers (gated on `json` feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "json")]
impl Theme {
    /// Deserialize a `Theme` from a JSON string.
    ///
    /// The JSON must be an object mapping style names to style definition
    /// strings, e.g. `{"info": "dim cyan", "warning": "bold yellow"}`.
    ///
    /// The resulting theme does **not** inherit from defaults — it contains
    /// only the styles listed in the JSON.  Call
    /// [`Theme::new(Some(styles), true)`](Theme::new) if you want inheritance.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "json")] {
    /// use gilt::theme::Theme;
    ///
    /// let theme = Theme::from_json_str(r#"{"info": "dim cyan"}"#).unwrap();
    /// assert!(theme.get("info").is_some());
    /// # }
    /// ```
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize a `Theme` from any [`std::io::Read`] source (e.g. a file).
    ///
    /// Convenience wrapper around [`serde_json::from_reader`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "json", not(target_arch = "wasm32")))] {
    /// use gilt::theme::Theme;
    ///
    /// let f = std::fs::File::open("theme.json").unwrap();
    /// let theme = Theme::from_json_reader(f).unwrap();
    /// # }
    /// ```
    pub fn from_json_reader<R: io::Read>(reader: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

// ---------------------------------------------------------------------------
// serde Serialize / Deserialize for Theme (gated on `json` feature)
// ---------------------------------------------------------------------------
//
// Theme serializes as a JSON object mapping style names to style strings,
// e.g. `{"bold": "bold", "warning": "bold red"}`. Deserialization builds
// a non-inheriting Theme from the map; callers that want inheritance must
// apply it after deserialization.

#[cfg(feature = "json")]
impl serde::Serialize for Theme {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(self.styles.len()))?;
        // Sort for deterministic output.
        let mut entries: Vec<(&String, &Style)> = self.styles.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        for (name, style) in entries {
            map.serialize_entry(name, &style.to_string())?;
        }
        map.end()
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for Theme {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{MapAccess, Visitor};
        struct ThemeVisitor;
        impl<'de> Visitor<'de> for ThemeVisitor {
            type Value = Theme;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a map of style name strings to style definition strings")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Theme, A::Error> {
                let mut styles = HashMap::new();
                while let Some((name, value)) = map.next_entry::<String, String>()? {
                    let style = Style::parse_strict(&value)
                        .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                    styles.insert(name, style);
                }
                Ok(Theme::new(Some(styles), false))
            }
        }
        d.deserialize_map(ThemeVisitor)
    }
}

/// Error returned when parsing a theme from a string fails.
#[derive(Debug)]
pub enum ThemeFromStrError {
    /// A line could not be parsed (missing `=`, empty name, etc.).
    Parse(String),
    /// A style definition could not be parsed by `Style::parse`.
    Style {
        /// The style name that failed.
        name: String,
        /// The underlying parse error.
        source: StyleError,
    },
}

impl fmt::Display for ThemeFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeFromStrError::Parse(msg) => write!(f, "theme parse error: {}", msg),
            ThemeFromStrError::Style { name, source } => {
                write!(f, "invalid style for '{}': {}", name, source)
            }
        }
    }
}

impl std::error::Error for ThemeFromStrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeFromStrError::Style { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Error returned when attempting to pop the base theme from a ThemeStack.
#[derive(Debug, Clone)]
pub struct ThemeStackError;

impl fmt::Display for ThemeStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to pop base theme")
    }
}

impl std::error::Error for ThemeStackError {}

/// A stack of theme style maps.
///
/// The ThemeStack allows pushing themed style overrides and popping them
/// to restore previous state. The base theme can never be popped.
pub struct ThemeStack {
    entries: Vec<HashMap<String, Style>>,
}

impl ThemeStack {
    /// Creates a new ThemeStack with the given theme as the base.
    pub fn new(theme: Theme) -> Self {
        ThemeStack {
            entries: vec![theme.styles],
        }
    }

    /// Looks up a style by name in the top-most theme on the stack.
    pub fn get(&self, name: &str) -> Option<&Style> {
        self.entries
            .last()
            .expect("ThemeStack should never be empty")
            .get(name)
    }

    /// Pushes a new theme onto the stack.
    ///
    /// If `inherit` is true (the default), the new layer inherits all styles
    /// from the current top of the stack, with the pushed theme's styles
    /// overriding any that share the same name.
    ///
    /// If `inherit` is false, only the pushed theme's styles are available.
    pub fn push_theme(&mut self, theme: Theme, inherit: bool) {
        let styles = if inherit {
            let mut merged = self
                .entries
                .last()
                .expect("ThemeStack should never be empty")
                .clone();
            merged.extend(theme.styles);
            merged
        } else {
            theme.styles
        };
        self.entries.push(styles);
    }

    /// Pops the top-most theme from the stack.
    ///
    /// Returns an error if attempting to pop the base theme.
    pub fn pop_theme(&mut self) -> Result<(), ThemeStackError> {
        if self.entries.len() == 1 {
            return Err(ThemeStackError);
        }
        self.entries.pop();
        Ok(())
    }
}

impl fmt::Debug for ThemeStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThemeStack")
            .field("depth", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_new_inherit() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), true);

        // Custom style is present
        assert_eq!(theme.get("warning").unwrap(), &Style::parse("red"));
        // Inherited default style is also present
        assert_eq!(theme.get("dim").unwrap(), &Style::parse("dim"));
    }

    #[test]
    fn test_theme_new_no_inherit() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), false);

        // Custom style is present
        assert!(theme.get("warning").is_some());
        // Default styles are NOT present
        assert!(theme.get("dim").is_none());
    }

    #[test]
    fn test_theme_new_none_styles() {
        let theme = Theme::new(None, true);
        // Should have all defaults
        assert_eq!(theme.styles.len(), DEFAULT_STYLES.len());
    }

    #[test]
    fn test_theme_config() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), true);
        let config = theme.config();
        assert!(config.starts_with("[styles]\n"));
        assert!(config.contains("warning = red\n"));
    }

    #[test]
    fn test_theme_config_sorted() {
        let mut custom = HashMap::new();
        custom.insert("zebra".to_string(), Style::parse("bold"));
        custom.insert("alpha".to_string(), Style::parse("dim"));
        let theme = Theme::new(Some(custom), false);
        let config = theme.config();
        let lines: Vec<&str> = config.lines().collect();
        // First line is [styles], then alpha, then zebra
        assert_eq!(lines[0], "[styles]");
        assert!(lines[1].starts_with("alpha"));
        assert!(lines[2].starts_with("zebra"));
    }

    #[test]
    fn test_theme_get_missing() {
        let theme = Theme::new(None, true);
        assert!(theme.get("nonexistent_style").is_none());
    }

    #[test]
    fn test_theme_stack_basic() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), true);
        let stack = ThemeStack::new(theme);
        assert_eq!(stack.get("warning").unwrap(), &Style::parse("red"));
    }

    #[test]
    fn test_theme_stack_push_pop() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), true);
        let mut stack = ThemeStack::new(theme);

        // Verify base style
        assert_eq!(stack.get("warning").unwrap(), &Style::parse("red"));

        // Push override
        let mut override_styles = HashMap::new();
        override_styles.insert("warning".to_string(), Style::parse("bold yellow"));
        let new_theme = Theme::new(Some(override_styles), false);
        stack.push_theme(new_theme, true);
        assert_eq!(stack.get("warning").unwrap(), &Style::parse("bold yellow"));

        // Pop restores original
        stack.pop_theme().unwrap();
        assert_eq!(stack.get("warning").unwrap(), &Style::parse("red"));
    }

    #[test]
    fn test_theme_stack_push_no_inherit() {
        let mut custom = HashMap::new();
        custom.insert("warning".to_string(), Style::parse("red"));
        let theme = Theme::new(Some(custom), true);
        let mut stack = ThemeStack::new(theme);

        let mut override_styles = HashMap::new();
        override_styles.insert("alert".to_string(), Style::parse("bold"));
        let new_theme = Theme::new(Some(override_styles), false);
        stack.push_theme(new_theme, false);

        // New style is present
        assert!(stack.get("alert").is_some());
        // Original styles are NOT present (no inherit)
        assert!(stack.get("warning").is_none());

        stack.pop_theme().unwrap();
        // After pop, original is back
        assert!(stack.get("warning").is_some());
    }

    #[test]
    fn test_pop_base_error() {
        let theme = Theme::new(None, true);
        let mut stack = ThemeStack::new(theme);
        let result = stack.pop_theme();
        assert!(result.is_err());
    }

    #[test]
    fn test_theme_stack_error_display() {
        let err = ThemeStackError;
        assert_eq!(err.to_string(), "Unable to pop base theme");
    }

    #[test]
    fn test_theme_stack_debug() {
        let theme = Theme::new(None, true);
        let stack = ThemeStack::new(theme);
        let debug = format!("{:?}", stack);
        assert!(debug.contains("ThemeStack"));
        assert!(debug.contains("depth"));
    }

    #[test]
    fn test_theme_stack_get_missing() {
        let theme = Theme::new(None, true);
        let stack = ThemeStack::new(theme);
        assert!(stack.get("nonexistent_style_xyz").is_none());
    }

    #[test]
    fn test_theme_override_default() {
        // Override a default style
        let mut custom = HashMap::new();
        custom.insert("bold".to_string(), Style::parse("italic"));
        let theme = Theme::new(Some(custom), true);
        // The "bold" name now maps to italic style
        assert_eq!(theme.get("bold").unwrap(), &Style::parse("italic"));
    }

    // ---- File-loading / INI parsing tests ----

    #[test]
    fn test_from_str_basic() {
        let content = "\
[styles]
info = dim cyan
warning = magenta
danger = bold red
";
        let theme = Theme::from_str(content, false).unwrap();
        assert_eq!(theme.get("info").unwrap(), &Style::parse("dim cyan"));
        assert_eq!(theme.get("warning").unwrap(), &Style::parse("magenta"));
        assert_eq!(theme.get("danger").unwrap(), &Style::parse("bold red"));
        // No inheritance — only the 3 styles we defined
        assert_eq!(theme.styles.len(), 3);
    }

    #[test]
    fn test_from_str_with_inheritance() {
        let content = "\
[styles]
info = dim cyan
";
        let theme = Theme::from_str(content, true).unwrap();
        // Our custom style is present
        assert_eq!(theme.get("info").unwrap(), &Style::parse("dim cyan"));
        // Inherited default styles are also present
        assert!(theme.get("dim").is_some());
        assert!(theme.styles.len() > 1);
    }

    #[test]
    fn test_from_str_comments_and_blanks() {
        let content = "\
# This is a comment
[styles]

# Another comment
info = dim cyan

warning = magenta
";
        let theme = Theme::from_str(content, false).unwrap();
        assert_eq!(theme.styles.len(), 2);
        assert!(theme.get("info").is_some());
        assert!(theme.get("warning").is_some());
    }

    #[test]
    fn test_from_str_ignores_non_styles_sections() {
        let content = "\
[metadata]
author = someone

[styles]
info = dim cyan
";
        let theme = Theme::from_str(content, false).unwrap();
        assert_eq!(theme.styles.len(), 1);
        assert!(theme.get("info").is_some());
        // "author" is not a style
        assert!(theme.get("author").is_none());
    }

    #[test]
    fn test_from_str_empty_content() {
        let theme = Theme::from_str("", false).unwrap();
        assert_eq!(theme.styles.len(), 0);
    }

    #[test]
    fn test_from_str_no_styles_section() {
        let content = "\
[metadata]
author = someone
";
        let theme = Theme::from_str(content, false).unwrap();
        assert_eq!(theme.styles.len(), 0);
    }

    #[test]
    fn test_from_str_invalid_style() {
        let content = "\
[styles]
bad_style = not_a_real_style_at_all zzz
";
        let result = Theme::from_str(content, false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match &err {
            ThemeFromStrError::Style { name, .. } => {
                assert_eq!(name, "bad_style");
            }
            other => panic!("expected Style error, got: {}", other),
        }
    }

    #[test]
    fn test_from_str_missing_equals() {
        let content = "\
[styles]
this has no equals sign
";
        let result = Theme::from_str(content, false);
        assert!(result.is_err());
        match &result.unwrap_err() {
            ThemeFromStrError::Parse(msg) => {
                assert!(msg.contains("expected 'name = style'"));
            }
            other => panic!("expected Parse error, got: {}", other),
        }
    }

    #[test]
    fn test_from_str_empty_name() {
        let content = "\
[styles]
 = bold red
";
        let result = Theme::from_str(content, false);
        assert!(result.is_err());
        match &result.unwrap_err() {
            ThemeFromStrError::Parse(msg) => {
                assert!(msg.contains("empty style name"));
            }
            other => panic!("expected Parse error, got: {}", other),
        }
    }

    #[test]
    fn test_to_config_alias() {
        let mut custom = HashMap::new();
        custom.insert("info".to_string(), Style::parse("cyan"));
        let theme = Theme::new(Some(custom), false);
        // to_config and config should return the same thing
        assert_eq!(theme.to_config(), theme.config());
    }

    #[test]
    fn test_round_trip() {
        // Create a theme, export it, re-import it, verify styles match
        let mut custom = HashMap::new();
        custom.insert("info".to_string(), Style::parse("dim cyan"));
        custom.insert("warning".to_string(), Style::parse("magenta"));
        custom.insert("danger".to_string(), Style::parse("bold red"));
        let original = Theme::new(Some(custom), false);

        let config = original.to_config();
        let restored = Theme::from_str(&config, false).unwrap();

        assert_eq!(original.styles.len(), restored.styles.len());
        for (name, style) in &original.styles {
            assert_eq!(
                restored.get(name).unwrap(),
                style,
                "style '{}' did not round-trip correctly",
                name
            );
        }
    }

    #[test]
    fn test_read_from_reader() {
        let content = "\
[styles]
info = dim cyan
warning = magenta
";
        let mut reader = std::io::Cursor::new(content);
        let theme = Theme::read(&mut reader, true).unwrap();
        assert_eq!(theme.get("info").unwrap(), &Style::parse("dim cyan"));
        assert_eq!(theme.get("warning").unwrap(), &Style::parse("magenta"));
    }

    #[test]
    fn test_from_file() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_theme.ini");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "[styles]\ninfo = dim cyan\nwarning = magenta\n").unwrap();
        }
        let theme = Theme::from_file(&path, true).unwrap();
        assert_eq!(theme.get("info").unwrap(), &Style::parse("dim cyan"));
        assert_eq!(theme.get("warning").unwrap(), &Style::parse("magenta"));
        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_from_file_not_found() {
        let result = Theme::from_file(Path::new("/nonexistent/path/theme.ini"), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_file_invalid_content() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_theme_bad.ini");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "[styles]\nbad line with no equals\n").unwrap();
        }
        let result = Theme::from_file(&path, true);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_theme_from_str_error_display() {
        let err = ThemeFromStrError::Parse("test message".to_string());
        assert_eq!(err.to_string(), "theme parse error: test message");
    }

    #[test]
    fn test_from_str_style_with_on_color() {
        let content = "\
[styles]
alert = bold red on white
";
        let theme = Theme::from_str(content, false).unwrap();
        assert_eq!(
            theme.get("alert").unwrap(),
            &Style::parse("bold red on white")
        );
    }

    #[test]
    fn test_from_str_dotted_names() {
        let content = "\
[styles]
bar.back = grey23
progress.elapsed = cyan
";
        let theme = Theme::from_str(content, false).unwrap();
        assert!(theme.get("bar.back").is_some());
        assert!(theme.get("progress.elapsed").is_some());
    }

    // ---- serde round-trip tests (json feature) ----------------------------

    #[cfg(feature = "json")]
    #[test]
    fn test_color_json_round_trip() {
        use crate::color::Color;

        // Named color
        let c = Color::parse("red").unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back, "red round-trip");

        // Hex / truecolor
        let c2 = Color::parse("#ff8800").unwrap();
        let json2 = serde_json::to_string(&c2).unwrap();
        let back2: Color = serde_json::from_str(&json2).unwrap();
        assert_eq!(c2, back2, "#ff8800 round-trip");

        // 8-bit color(123)
        let c3 = Color::parse("color(123)").unwrap();
        let json3 = serde_json::to_string(&c3).unwrap();
        let back3: Color = serde_json::from_str(&json3).unwrap();
        assert_eq!(c3, back3, "color(123) round-trip");

        // Default
        let c4 = Color::default_color();
        let json4 = serde_json::to_string(&c4).unwrap();
        let back4: Color = serde_json::from_str(&json4).unwrap();
        assert_eq!(c4, back4, "default round-trip");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_style_json_round_trip() {
        use crate::style::Style;

        let cases = [
            "bold red",
            "dim italic on blue",
            "none",
            "bold red on #282a36",
        ];
        for case in &cases {
            let style = Style::parse(case);
            let json = serde_json::to_string(&style).unwrap();
            let back: Style = serde_json::from_str(&json).unwrap();
            assert_eq!(style, back, "style '{}' did not round-trip", case);
        }
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_theme_json_round_trip() {
        let mut custom = HashMap::new();
        custom.insert("info".to_string(), Style::parse("dim cyan"));
        custom.insert("warning".to_string(), Style::parse("bold yellow"));
        custom.insert("danger".to_string(), Style::parse("bold red"));
        let original = Theme::new(Some(custom), false);

        let json = serde_json::to_string(&original).unwrap();
        let back: Theme = serde_json::from_str(&json).unwrap();

        assert_eq!(original.styles.len(), back.styles.len());
        for (name, style) in &original.styles {
            assert_eq!(
                back.get(name).unwrap(),
                style,
                "style '{}' did not round-trip",
                name
            );
        }
    }

    #[test]
    fn test_semicolon_comment_skipped() {
        let content = "[styles]\n; this is a semicolon comment\ninfo = dim cyan\n";
        let theme = Theme::from_str(content, false).unwrap();
        assert!(theme.get("info").is_some());
    }

    #[test]
    fn test_from_file_inherit_false() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_theme_noinherit.ini");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "[styles]\ninfo = dim cyan\n").unwrap();
        }
        let theme = Theme::from_file(&path, false).unwrap();
        assert!(theme.get("info").is_some());
        assert!(theme.get("dim").is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_read_inherit_false() {
        let content = "[styles]\ninfo = dim cyan\n";
        let mut reader = std::io::Cursor::new(content);
        let theme = Theme::read(&mut reader, false).unwrap();
        assert!(theme.get("info").is_some());
        assert!(theme.get("dim").is_none());
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_theme_json_serializes_as_map() {
        let mut custom = HashMap::new();
        custom.insert("alert".to_string(), Style::parse("bold red"));
        let theme = Theme::new(Some(custom), false);
        let json = serde_json::to_string(&theme).unwrap();
        // Must be a JSON object
        assert!(json.starts_with('{'));
        assert!(json.contains("\"alert\""));
        assert!(json.contains("bold red") || json.contains("red bold"));
    }
}
