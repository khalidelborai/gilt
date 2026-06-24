//! Text column for progress bars.

use crate::progress::{ProgressColumn, Task};
use crate::style::Style;
use crate::text::{JustifyMethod, Text};

/// A column that renders text with simple template substitution.
///
/// Supported placeholders:
/// - `{task.description}` - task description
/// - `{task.percentage:.0f}` (or any format) - percentage complete
/// - `{task.completed}` - completed count
/// - `{task.total}` - total count (or "?" if None)
/// - `{task.speed}` - current speed (or "?" if unknown)
///
/// Any field key `{task.fields.KEY}` substitutes the corresponding
/// entry from `task.fields`.
#[derive(Debug, Clone)]
pub struct TextColumn {
    /// Template string with `{task.*}` placeholders.
    pub text: String,
    /// Style applied to the rendered text.
    pub style: Option<Style>,
    /// Horizontal justification.
    pub justify: JustifyMethod,
}

impl TextColumn {
    /// Create a new TextColumn with the given template.
    pub fn new(text: &str) -> Self {
        TextColumn {
            text: text.to_string(),
            style: None,
            justify: JustifyMethod::Left,
        }
    }

    /// Builder: set the style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: set justification.
    #[must_use]
    pub fn with_justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = justify;
        self
    }

    /// Perform template substitution for a task.
    fn substitute(&self, task: &Task) -> String {
        let mut result = self.text.clone();

        // {task.description}
        result = result.replace("{task.description}", &task.description);

        // {task.percentage} and {task.percentage:.Nf}
        if result.contains("{task.percentage") {
            let pct = task.percentage();
            // Handle format specifiers like {task.percentage:.0f}
            if let Some(start) = result.find("{task.percentage:") {
                if let Some(end) = result[start..].find('}') {
                    let spec = &result[start..start + end + 1];
                    // Parse precision from :.Nf pattern
                    let formatted = if spec.contains(".0f") {
                        format!("{pct:.0}")
                    } else if spec.contains(".1f") {
                        format!("{pct:.1}")
                    } else if spec.contains(".2f") {
                        format!("{pct:.2}")
                    } else {
                        format!("{pct:.1}")
                    };
                    result = result.replace(spec, &formatted);
                }
            }
            result = result.replace("{task.percentage}", &format!("{pct:.1}"));
        }

        // {task.completed}
        result = result.replace("{task.completed}", &format!("{}", task.completed));

        // {task.total}
        let total_str = match task.total {
            Some(t) => format!("{t}"),
            None => "?".to_string(),
        };
        result = result.replace("{task.total}", &total_str);

        // {task.speed}
        let speed_str = match task.speed() {
            Some(s) => format!("{s:.1}"),
            None => "?".to_string(),
        };
        result = result.replace("{task.speed}", &speed_str);

        // {task.fields.KEY}
        for (key, value) in &task.fields {
            let placeholder = format!("{{task.fields.{key}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }
}

impl ProgressColumn for TextColumn {
    fn render(&self, task: &Task) -> Text {
        let content = self.substitute(task);
        let style = self.style.clone().unwrap_or_else(Style::null);
        // Attempt to parse markup tags so [bold]text[/bold]-style syntax in
        // templates is honoured.  Fall back to a plain Text::new when the
        // content is not valid markup (rich parity — P2 fix).
        let mut text =
            Text::from_markup(&content).unwrap_or_else(|_| Text::new(&content, Style::null()));
        // Apply the column-level base style unconditionally (rich parity: Text.stylize(style)).
        // Use stylize_before so it underlies any markup spans that came from parsing.
        if !style.is_null() {
            text.stylize_before(style, 0, None);
        }
        text.justify = Some(self.justify);
        text
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// TextColumn must parse markup in the substituted string.
    #[test]
    fn text_column_parses_markup() {
        let mut task = Task::new(0, "hello", Some(100.0));
        task.fields.insert("msg".to_string(), "world".to_string());
        // Template with markup tags — the plain content should be "hello"
        // regardless of the markup tags wrapping it.
        let col = TextColumn::new("[bold]{task.description}[/bold]");
        let text = col.render(&task);
        // plain() strips markup; the content must still be "hello".
        assert_eq!(
            text.plain(),
            "hello",
            "markup should be parsed, plain text preserved"
        );
    }

    /// Invalid markup falls back to plain text without panicking.
    #[test]
    fn text_column_invalid_markup_falls_back_gracefully() {
        let task = Task::new(0, "oops [unclosed", Some(10.0));
        let col = TextColumn::new("{task.description}");
        // Must not panic; plain output should contain the description.
        let text = col.render(&task);
        assert!(
            text.plain().contains("oops"),
            "fallback should preserve plain text, got: {}",
            text.plain()
        );
    }

    /// Column-level style must be applied even when markup parsing succeeds
    /// (rich parity: TextColumn.stylize(style) is always called).
    #[test]
    fn text_column_applies_base_style_on_markup_success() {
        let task = Task::new(0, "hello world", Some(100.0));
        let bold = Style::parse("bold");
        let col = TextColumn::new("{task.description}").with_style(bold.clone());
        let text = col.render(&task);
        // The plain text must be correct.
        assert_eq!(text.plain(), "hello world");
        // The base bold style must appear as a span.
        let spans = text.spans();
        assert!(
            !spans.is_empty(),
            "expected at least one span carrying the column style, got none"
        );
        let has_bold = spans.iter().any(|s| s.style == bold);
        assert!(
            has_bold,
            "expected a span with bold style, got spans: {:?}",
            spans
        );
    }

    /// Column-level style must also be applied when the fallback path fires
    /// (invalid markup).
    #[test]
    fn text_column_applies_base_style_on_markup_failure() {
        let task = Task::new(0, "oops [unclosed", Some(10.0));
        let bold = Style::parse("bold");
        let col = TextColumn::new("{task.description}").with_style(bold.clone());
        let text = col.render(&task);
        assert!(text.plain().contains("oops"));
        let has_bold = text.spans().iter().any(|s| s.style == bold);
        assert!(
            has_bold,
            "expected bold span in fallback path, got spans: {:?}",
            text.spans()
        );
    }
}
