//! Breadcrumbs widget for showing navigation path/location.
//!
//! Breadcrumbs display a hierarchical path, useful for navigation in multi-step flows,
//! file system paths, or showing the user's current location in an application hierarchy.
//!
//! # Examples
//!
//! ```
//! use gilt::breadcrumbs::Breadcrumbs;
//!
//! // Create breadcrumbs from a path string
//! let crumbs = Breadcrumbs::from_path("Home/Settings/Profile");
//!
//! // Create breadcrumbs from items with custom separator
//! let crumbs = Breadcrumbs::new(vec![
//!     "Home".into(),
//!     "Products".into(),
//!     "Electronics".into(),
//!     "Laptops".into(),
//! ])
//! .separator(" / ");
//!
//! // Use convenience constructors for common separators
//! let crumbs = Breadcrumbs::slash(vec!["docs".into(), "api".into(), "v1".into()]);
//! let crumbs = Breadcrumbs::arrow(vec!["Start".into(), "Middle".into(), "End".into()]);
//! let crumbs = Breadcrumbs::chevron(vec!["First".into(), "Second".into(), "Third".into()]);
//! ```

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;

/// A breadcrumbs widget for showing navigation path.
///
/// Breadcrumbs display a sequence of items separated by a delimiter,
/// typically used to show the user's current location within a hierarchy.
/// The last item can be styled differently to indicate it's the active/current location.
///
/// # Examples
///
/// ```
/// use gilt::breadcrumbs::Breadcrumbs;
/// use gilt::style::Style;
///
/// // Basic usage with default separator
/// let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into(), "Profile".into()]);
///
/// // Styled breadcrumbs with custom active style
/// let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into(), "C".into()])
///     .style(Style::null())
///     .separator_style(Style::parse("dim").unwrap())
///     .active_style(Style::parse("bold green").unwrap());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumbs {
    items: Vec<String>,
    separator: String,
    style: Style,
    separator_style: Style,
    active_style: Option<Style>,
}

impl Breadcrumbs {
    /// Create breadcrumbs from a path string (split on `/`).
    ///
    /// The path is split on `/` characters, and empty components are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::from_path("Home/Settings/Profile");
    /// assert_eq!(crumbs.len(), 3);
    /// ```
    pub fn from_path(path: &str) -> Self {
        let items: Vec<String> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Self::new(items)
    }

    /// Create breadcrumbs from a vector of items.
    ///
    /// Defaults to ` > ` as the separator, null style for items,
    /// dim style for separators, and no special active style.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into()]);
    /// ```
    pub fn new(items: Vec<String>) -> Self {
        Breadcrumbs {
            items,
            separator: " > ".to_string(),
            style: Style::null(),
            separator_style: Style::parse("dim").unwrap_or_else(|_| Style::null()),
            active_style: None,
        }
    }

    /// Set the separator string.
    ///
    /// Default is ` > `.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into()]).separator(" → ");
    /// ```
    #[must_use]
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Set the base style for breadcrumb items.
    ///
    /// This style is applied to all items except the last one (if active_style is set).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    /// use gilt::style::Style;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into()])
    ///     .style(Style::parse("blue").unwrap());
    /// ```
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the style for separators.
    ///
    /// Default is `dim`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    /// use gilt::style::Style;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into()])
    ///     .separator_style(Style::parse("yellow").unwrap());
    /// ```
    #[must_use]
    pub fn separator_style(mut self, style: Style) -> Self {
        self.separator_style = style;
        self
    }

    /// Set the style for the active (last) item.
    ///
    /// When set, the last item in the breadcrumbs will be styled with this style
    /// instead of the base style.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    /// use gilt::style::Style;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["Home".into(), "Profile".into()])
    ///     .active_style(Style::parse("bold green").unwrap());
    /// ```
    #[must_use]
    pub fn active_style(mut self, style: Style) -> Self {
        self.active_style = Some(style);
        self
    }

    /// Add an item to the end of the breadcrumbs.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let mut crumbs = Breadcrumbs::new(vec!["Home".into()]);
    /// crumbs.push("Settings");
    /// crumbs.push("Profile");
    /// assert_eq!(crumbs.len(), 3);
    /// ```
    pub fn push(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// Remove and return the last item, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let mut crumbs = Breadcrumbs::new(vec!["A".into(), "B".into(), "C".into()]);
    /// let last = crumbs.pop();
    /// assert_eq!(last, Some("C".to_string()));
    /// assert_eq!(crumbs.len(), 2);
    /// ```
    pub fn pop(&mut self) -> Option<String> {
        self.items.pop()
    }

    /// Convenience: create breadcrumbs with slash separator (` / `).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::slash(vec!["usr".into(), "local".into(), "bin".into()]);
    /// // Renders as: usr / local / bin
    /// ```
    pub fn slash(items: Vec<String>) -> Self {
        Self::new(items).separator(" / ")
    }

    /// Convenience: create breadcrumbs with arrow separator (` → `).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::arrow(vec!["Step 1".into(), "Step 2".into(), "Step 3".into()]);
    /// // Renders as: Step 1 → Step 2 → Step 3
    /// ```
    pub fn arrow(items: Vec<String>) -> Self {
        Self::new(items).separator(" → ")
    }

    /// Convenience: create breadcrumbs with chevron separator (` › `).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::chevron(vec!["First".into(), "Second".into(), "Third".into()]);
    /// // Renders as: First › Second › Third
    /// ```
    pub fn chevron(items: Vec<String>) -> Self {
        Self::new(items).separator(" › ")
    }

    /// Get the number of items in the breadcrumbs.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into(), "C".into()]);
    /// assert_eq!(crumbs.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the breadcrumbs are empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::new(vec![]);
    /// assert!(crumbs.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the items in the breadcrumbs.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::breadcrumbs::Breadcrumbs;
    ///
    /// let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into()]);
    /// assert_eq!(crumbs.items(), &["Home", "Settings"]);
    /// ```
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Get the separator string.
    pub fn separator_str(&self) -> &str {
        &self.separator
    }

    /// Get the base style.
    pub fn base_style(&self) -> &Style {
        &self.style
    }

    /// Get the separator style.
    pub fn sep_style(&self) -> &Style {
        &self.separator_style
    }

    /// Get the active style, if set.
    pub fn active_style_opt(&self) -> Option<&Style> {
        self.active_style.as_ref()
    }
}

impl Renderable for Breadcrumbs {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();

        let item_count = self.items.len();

        for (i, item) in self.items.iter().enumerate() {
            let is_last = i == item_count.saturating_sub(1);

            // Determine the style for this item
            let item_style = if is_last {
                self.active_style
                    .clone()
                    .unwrap_or_else(|| self.style.clone())
            } else {
                self.style.clone()
            };

            // Add the item text
            segments.push(Segment::styled(item, item_style));

            // Add separator if not the last item
            if !is_last {
                segments.push(Segment::styled(
                    &self.separator,
                    self.separator_style.clone(),
                ));
            }
        }

        // Add trailing newline
        segments.push(Segment::line());

        segments
    }
}

impl std::fmt::Display for Breadcrumbs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn render_breadcrumbs(console: &Console, breadcrumbs: &Breadcrumbs) -> String {
        let opts = console.options();
        let segments = breadcrumbs.gilt_console(console, &opts);
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction ---------------------------------------------------------

    #[test]
    fn test_new() {
        let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into()]);
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs.items(), &["Home", "Settings"]);
        assert_eq!(crumbs.separator_str(), " > ");
    }

    #[test]
    fn test_from_path() {
        let crumbs = Breadcrumbs::from_path("Home/Settings/Profile");
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs.items(), &["Home", "Settings", "Profile"]);
    }

    #[test]
    fn test_from_path_with_empty_components() {
        let crumbs = Breadcrumbs::from_path("/Home//Settings/");
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs.items(), &["Home", "Settings"]);
    }

    #[test]
    fn test_empty() {
        let crumbs = Breadcrumbs::new(vec![]);
        assert!(crumbs.is_empty());
        assert_eq!(crumbs.len(), 0);
    }

    // -- Convenience constructors ---------------------------------------------

    #[test]
    fn test_slash() {
        let crumbs = Breadcrumbs::slash(vec!["usr".into(), "local".into(), "bin".into()]);
        assert_eq!(crumbs.separator_str(), " / ");
        assert_eq!(crumbs.items(), &["usr", "local", "bin"]);
    }

    #[test]
    fn test_arrow() {
        let crumbs = Breadcrumbs::arrow(vec!["A".into(), "B".into()]);
        assert_eq!(crumbs.separator_str(), " → ");
    }

    #[test]
    fn test_chevron() {
        let crumbs = Breadcrumbs::chevron(vec!["A".into(), "B".into()]);
        assert_eq!(crumbs.separator_str(), " › ");
    }

    // -- Builder pattern ------------------------------------------------------

    #[test]
    fn test_builder_separator() {
        let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into()]).separator(" | ");
        assert_eq!(crumbs.separator_str(), " | ");
    }

    #[test]
    fn test_builder_style() {
        let style = Style::parse("blue").unwrap();
        let crumbs = Breadcrumbs::new(vec!["A".into()]).style(style.clone());
        assert_eq!(crumbs.base_style(), &style);
    }

    #[test]
    fn test_builder_separator_style() {
        let style = Style::parse("yellow").unwrap();
        let crumbs = Breadcrumbs::new(vec!["A".into()]).separator_style(style.clone());
        assert_eq!(crumbs.sep_style(), &style);
    }

    #[test]
    fn test_builder_active_style() {
        let style = Style::parse("bold").unwrap();
        let crumbs = Breadcrumbs::new(vec!["A".into()]).active_style(style.clone());
        assert_eq!(crumbs.active_style_opt(), Some(&style));
    }

    #[test]
    fn test_builder_chain() {
        let crumbs = Breadcrumbs::new(vec!["Home".into(), "Profile".into()])
            .separator(" / ")
            .style(Style::parse("white").unwrap())
            .separator_style(Style::parse("dim").unwrap())
            .active_style(Style::parse("bold green").unwrap());

        assert_eq!(crumbs.separator_str(), " / ");
        assert_eq!(crumbs.len(), 2);
    }

    // -- Push and Pop ---------------------------------------------------------

    #[test]
    fn test_push() {
        let mut crumbs = Breadcrumbs::new(vec!["Home".into()]);
        crumbs.push("Settings");
        crumbs.push("Profile");
        assert_eq!(crumbs.len(), 3);
        assert_eq!(crumbs.items(), &["Home", "Settings", "Profile"]);
    }

    #[test]
    fn test_push_string() {
        let mut crumbs = Breadcrumbs::new(vec![]);
        crumbs.push("test".to_string());
        assert_eq!(crumbs.len(), 1);
    }

    #[test]
    fn test_pop() {
        let mut crumbs = Breadcrumbs::new(vec!["A".into(), "B".into(), "C".into()]);
        let last = crumbs.pop();
        assert_eq!(last, Some("C".to_string()));
        assert_eq!(crumbs.len(), 2);
    }

    #[test]
    fn test_pop_empty() {
        let mut crumbs = Breadcrumbs::new(vec![]);
        let last = crumbs.pop();
        assert_eq!(last, None);
    }

    #[test]
    fn test_push_pop_sequence() {
        let mut crumbs = Breadcrumbs::new(vec!["Home".into()]);
        crumbs.push("Settings");
        assert_eq!(crumbs.pop(), Some("Settings".to_string()));
        assert_eq!(crumbs.pop(), Some("Home".to_string()));
        assert_eq!(crumbs.pop(), None);
    }

    // -- Rendering ------------------------------------------------------------

    #[test]
    fn test_render_single_item() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::new(vec!["Home".into()]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("Home"));
        assert!(!output.contains(" > ")); // No separator for single item
    }

    #[test]
    fn test_render_multiple_items() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into(), "Profile".into()]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("Home"));
        assert!(output.contains("Settings"));
        assert!(output.contains("Profile"));
        assert!(output.contains(" > "));
        // Should have 2 separators for 3 items
        let separator_count = output.matches(" > ").count();
        assert_eq!(separator_count, 2);
    }

    #[test]
    fn test_render_with_custom_separator() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into(), "C".into()]).separator(" / ");
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("A / B / C"));
    }

    #[test]
    fn test_render_slash() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::slash(vec!["usr".into(), "local".into(), "bin".into()]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("usr / local / bin"));
    }

    #[test]
    fn test_render_arrow() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::arrow(vec!["A".into(), "B".into()]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("A → B"));
    }

    #[test]
    fn test_render_chevron() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::chevron(vec!["A".into(), "B".into()]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("A › B"));
    }

    #[test]
    fn test_render_empty() {
        let console = make_console(80);
        let crumbs = Breadcrumbs::new(vec![]);
        let output = render_breadcrumbs(&console, &crumbs);

        assert_eq!(output.trim(), "");
    }

    // -- Display trait --------------------------------------------------------

    #[test]
    fn test_display_trait() {
        let crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into()]);
        let s = format!("{}", crumbs);
        assert!(s.contains("Home"));
        assert!(s.contains("Settings"));
        assert!(s.contains(" > "));
    }

    #[test]
    fn test_display_single_item() {
        let crumbs = Breadcrumbs::new(vec!["Home".into()]);
        let s = format!("{}", crumbs);
        assert_eq!(s, "Home");
    }

    // -- Clone ----------------------------------------------------------------

    #[test]
    fn test_clone() {
        let crumbs = Breadcrumbs::new(vec!["A".into(), "B".into()])
            .separator(" / ")
            .active_style(Style::parse("bold").unwrap());

        let cloned = crumbs.clone();
        assert_eq!(crumbs.items(), cloned.items());
        assert_eq!(crumbs.separator_str(), cloned.separator_str());
    }

    // -- Real-world use cases -------------------------------------------------

    #[test]
    fn test_file_path_navigation() {
        let crumbs = Breadcrumbs::slash(vec![
            "home".into(),
            "user".into(),
            "projects".into(),
            "myapp".into(),
            "src".into(),
        ]);

        let console = make_console(80);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("home / user / projects / myapp / src"));
    }

    #[test]
    fn test_navigation_flow() {
        let crumbs = Breadcrumbs::new(vec![
            "Dashboard".into(),
            "Users".into(),
            "User Details".into(),
        ])
        .active_style(Style::parse("bold").unwrap());

        let console = make_console(80);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("Dashboard"));
        assert!(output.contains("Users"));
        assert!(output.contains("User Details"));
    }

    #[test]
    fn test_settings_hierarchy() {
        let crumbs = Breadcrumbs::chevron(vec![
            "Application".into(),
            "Preferences".into(),
            "Display".into(),
        ]);

        let console = make_console(80);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("Application › Preferences › Display"));
    }

    #[test]
    fn test_wizard_steps() {
        let crumbs = Breadcrumbs::arrow(vec![
            "Welcome".into(),
            "Configuration".into(),
            "Review".into(),
            "Complete".into(),
        ]);

        let console = make_console(80);
        let output = render_breadcrumbs(&console, &crumbs);

        assert!(output.contains("Welcome → Configuration → Review → Complete"));
    }
}
