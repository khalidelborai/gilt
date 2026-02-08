//! Default style definitions for the gilt library.
//!
//! This module provides a comprehensive set of 153 named styles that map to
//! the default styles in Python's rich library. These styles are used by
//! various components for consistent terminal formatting.

use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::style::Style;

/// Helper: insert a parsed style into a map.
fn ins(m: &mut HashMap<String, Style>, name: &str, def: &str) {
    m.insert(
        name.to_string(),
        Style::parse(def).unwrap_or_else(|e| {
            panic!(
                "Failed to parse default style '{}' = '{}': {}",
                name, def, e
            )
        }),
    );
}

/// Helper: insert a null (empty) style into a map.
fn null(m: &mut HashMap<String, Style>, name: &str) {
    m.insert(name.to_string(), Style::null());
}

/// The complete set of 153 default named styles.
///
/// Styles are lazily initialized on first access and cached for the lifetime
/// of the program.
pub static DEFAULT_STYLES: Lazy<HashMap<String, Style>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // --- Basic styles ---
    null(&mut m, "none");

    // "reset" needs explicit construction: color=default, bgcolor=default, all attrs=false
    m.insert(
        "reset".to_string(),
        Style::new(
            Some("default"),
            Some("default"),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            Some(false),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("reset style"),
    );

    ins(&mut m, "dim", "dim");
    ins(&mut m, "bright", "not dim");
    ins(&mut m, "bold", "bold");
    ins(&mut m, "strong", "bold");
    ins(&mut m, "code", "reverse bold");
    ins(&mut m, "italic", "italic");
    ins(&mut m, "emphasize", "italic");
    ins(&mut m, "underline", "underline");
    ins(&mut m, "blink", "blink");
    ins(&mut m, "blink2", "blink2");
    ins(&mut m, "reverse", "reverse");
    ins(&mut m, "strike", "strike");

    // --- Named colors ---
    ins(&mut m, "black", "black");
    ins(&mut m, "red", "red");
    ins(&mut m, "green", "green");
    ins(&mut m, "yellow", "yellow");
    ins(&mut m, "magenta", "magenta");
    ins(&mut m, "cyan", "cyan");
    ins(&mut m, "white", "white");

    // --- inspect.* styles ---
    ins(&mut m, "inspect.attr", "italic yellow");
    ins(&mut m, "inspect.attr.dunder", "italic dim yellow");
    ins(&mut m, "inspect.callable", "bold red");
    ins(&mut m, "inspect.async_def", "italic bright_cyan");
    ins(&mut m, "inspect.def", "italic bright_cyan");
    ins(&mut m, "inspect.class", "italic bright_cyan");
    ins(&mut m, "inspect.error", "bold red");
    null(&mut m, "inspect.equals");
    ins(&mut m, "inspect.help", "cyan");
    ins(&mut m, "inspect.doc", "dim");
    ins(&mut m, "inspect.value.border", "green");

    // --- live.* styles ---
    ins(&mut m, "live.ellipsis", "bold red");

    // --- layout.* styles ---
    ins(&mut m, "layout.tree.row", "not dim red");
    ins(&mut m, "layout.tree.column", "not dim blue");

    // --- logging.* styles ---
    ins(&mut m, "logging.keyword", "bold yellow");
    ins(&mut m, "logging.level.notset", "dim");
    ins(&mut m, "logging.level.debug", "green");
    ins(&mut m, "logging.level.info", "blue");
    ins(&mut m, "logging.level.warning", "yellow");
    ins(&mut m, "logging.level.error", "bold red");
    ins(&mut m, "logging.level.critical", "bold reverse red");

    // --- log.* styles ---
    null(&mut m, "log.level");
    ins(&mut m, "log.time", "dim cyan");
    null(&mut m, "log.message");
    ins(&mut m, "log.path", "dim");

    // --- repr.* styles ---
    ins(&mut m, "repr.ellipsis", "yellow");
    ins(&mut m, "repr.indent", "dim green");
    ins(&mut m, "repr.error", "bold red");
    ins(&mut m, "repr.str", "not italic not bold green");
    ins(&mut m, "repr.brace", "bold");
    ins(&mut m, "repr.comma", "bold");
    ins(&mut m, "repr.ipv4", "bold bright_green");
    ins(&mut m, "repr.ipv6", "bold bright_green");
    ins(&mut m, "repr.eui48", "bold bright_green");
    ins(&mut m, "repr.eui64", "bold bright_green");
    ins(&mut m, "repr.tag_start", "bold");
    ins(&mut m, "repr.tag_name", "bold bright_magenta");
    ins(&mut m, "repr.tag_contents", "default");
    ins(&mut m, "repr.tag_end", "bold");
    ins(&mut m, "repr.attrib_name", "not italic yellow");
    ins(&mut m, "repr.attrib_equal", "bold");
    ins(&mut m, "repr.attrib_value", "not italic magenta");
    ins(&mut m, "repr.number", "bold not italic cyan");
    ins(&mut m, "repr.number_complex", "bold not italic cyan");
    ins(&mut m, "repr.bool_true", "italic bright_green");
    ins(&mut m, "repr.bool_false", "italic bright_red");
    ins(&mut m, "repr.none", "italic magenta");
    ins(
        &mut m,
        "repr.url",
        "underline not italic not bold bright_blue",
    );
    ins(&mut m, "repr.uuid", "not bold bright_yellow");
    ins(&mut m, "repr.call", "bold magenta");
    ins(&mut m, "repr.path", "magenta");
    ins(&mut m, "repr.filename", "bright_magenta");

    // --- rule.* styles ---
    ins(&mut m, "rule.line", "bright_green");
    null(&mut m, "rule.text");

    // --- json.* styles ---
    ins(&mut m, "json.brace", "bold");
    ins(&mut m, "json.bool_true", "italic bright_green");
    ins(&mut m, "json.bool_false", "italic bright_red");
    ins(&mut m, "json.null", "italic magenta");
    ins(&mut m, "json.number", "bold not italic cyan");
    ins(&mut m, "json.str", "not italic not bold green");
    ins(&mut m, "json.key", "bold blue");

    // --- prompt.* styles ---
    null(&mut m, "prompt");
    ins(&mut m, "prompt.choices", "bold magenta");
    ins(&mut m, "prompt.default", "bold cyan");
    ins(&mut m, "prompt.invalid", "red");
    ins(&mut m, "prompt.invalid.choice", "red");

    // --- pretty ---
    null(&mut m, "pretty");

    // --- scope.* styles ---
    ins(&mut m, "scope.border", "blue");
    ins(&mut m, "scope.key", "italic yellow");
    ins(&mut m, "scope.key.special", "italic dim yellow");
    ins(&mut m, "scope.equals", "red");

    // --- table.* styles ---
    ins(&mut m, "table.header", "bold");
    ins(&mut m, "table.footer", "bold");
    null(&mut m, "table.cell");
    ins(&mut m, "table.title", "italic");
    ins(&mut m, "table.caption", "italic dim");

    // --- traceback.* styles ---
    ins(&mut m, "traceback.error", "italic red");
    ins(&mut m, "traceback.border.syntax_error", "bright_red");
    ins(&mut m, "traceback.border", "red");
    null(&mut m, "traceback.text");
    ins(&mut m, "traceback.title", "bold red");
    ins(&mut m, "traceback.exc_type", "bold bright_red");
    null(&mut m, "traceback.exc_value");
    ins(&mut m, "traceback.offset", "bold bright_red");
    ins(&mut m, "traceback.error_range", "underline bold");
    ins(&mut m, "traceback.note", "bold green");
    ins(&mut m, "traceback.group.border", "magenta");

    // --- bar.* styles ---
    ins(&mut m, "bar.back", "grey23");
    ins(&mut m, "bar.complete", "rgb(249,38,114)");
    ins(&mut m, "bar.finished", "rgb(114,156,31)");
    ins(&mut m, "bar.pulse", "rgb(249,38,114)");

    // --- progress.* styles ---
    null(&mut m, "progress.description");
    ins(&mut m, "progress.filesize", "green");
    ins(&mut m, "progress.filesize.total", "green");
    ins(&mut m, "progress.download", "green");
    ins(&mut m, "progress.elapsed", "yellow");
    ins(&mut m, "progress.percentage", "magenta");
    ins(&mut m, "progress.remaining", "cyan");
    ins(&mut m, "progress.data.speed", "red");
    ins(&mut m, "progress.spinner", "green");

    // --- status.* styles ---
    ins(&mut m, "status.spinner", "green");

    // --- tree styles ---
    null(&mut m, "tree");
    null(&mut m, "tree.line");

    // --- markdown.* styles ---
    null(&mut m, "markdown.paragraph");
    null(&mut m, "markdown.text");
    ins(&mut m, "markdown.em", "italic");
    ins(&mut m, "markdown.emph", "italic");
    ins(&mut m, "markdown.strong", "bold");
    ins(&mut m, "markdown.code", "bold cyan on black");
    ins(&mut m, "markdown.code_block", "cyan on black");
    ins(&mut m, "markdown.block_quote", "magenta");
    ins(&mut m, "markdown.list", "cyan");
    null(&mut m, "markdown.item");
    ins(&mut m, "markdown.item.bullet", "bold");
    ins(&mut m, "markdown.item.number", "cyan");
    ins(&mut m, "markdown.hr", "dim");
    null(&mut m, "markdown.h1.border");
    ins(&mut m, "markdown.h1", "bold underline");
    ins(&mut m, "markdown.h2", "underline magenta");
    ins(&mut m, "markdown.h3", "bold magenta");
    ins(&mut m, "markdown.h4", "italic magenta");
    ins(&mut m, "markdown.h5", "italic");
    ins(&mut m, "markdown.h6", "dim");
    ins(&mut m, "markdown.h7", "italic dim");
    ins(&mut m, "markdown.link", "bright_blue");
    ins(&mut m, "markdown.link_url", "underline blue");
    ins(&mut m, "markdown.s", "strike");
    ins(&mut m, "markdown.table.border", "cyan");
    ins(&mut m, "markdown.table.header", "not bold cyan");

    // --- iso8601.* styles ---
    ins(&mut m, "iso8601.date", "blue");
    ins(&mut m, "iso8601.time", "magenta");
    ins(&mut m, "iso8601.timezone", "yellow");

    m
});

/// Returns a clone of the default styles map.
pub fn default_styles() -> HashMap<String, Style> {
    DEFAULT_STYLES.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_styles_count() {
        assert_eq!(DEFAULT_STYLES.len(), 153);
    }

    #[test]
    fn test_none_is_null() {
        let style = DEFAULT_STYLES.get("none").unwrap();
        assert!(style.is_null());
    }

    #[test]
    fn test_reset_style() {
        let style = DEFAULT_STYLES.get("reset").unwrap();
        assert_eq!(style.bold(), Some(false));
        assert_eq!(style.dim(), Some(false));
        assert_eq!(style.italic(), Some(false));
        assert_eq!(style.underline(), Some(false));
        assert_eq!(style.blink(), Some(false));
        assert_eq!(style.blink2(), Some(false));
        assert_eq!(style.reverse(), Some(false));
        assert_eq!(style.conceal(), Some(false));
        assert_eq!(style.strike(), Some(false));
        assert!(style.color().is_some());
        assert!(style.bgcolor().is_some());
    }

    #[test]
    fn test_dim_style() {
        let style = DEFAULT_STYLES.get("dim").unwrap();
        assert_eq!(style.dim(), Some(true));
    }

    #[test]
    fn test_bright_style() {
        let style = DEFAULT_STYLES.get("bright").unwrap();
        assert_eq!(style.dim(), Some(false));
    }

    #[test]
    fn test_bold_style() {
        let style = DEFAULT_STYLES.get("bold").unwrap();
        assert_eq!(style.bold(), Some(true));
    }

    #[test]
    fn test_code_style() {
        let style = DEFAULT_STYLES.get("code").unwrap();
        assert_eq!(style.reverse(), Some(true));
        assert_eq!(style.bold(), Some(true));
    }

    #[test]
    fn test_color_styles() {
        let style = DEFAULT_STYLES.get("red").unwrap();
        assert_eq!(style.color().unwrap().name, "red");

        let style = DEFAULT_STYLES.get("green").unwrap();
        assert_eq!(style.color().unwrap().name, "green");
    }

    #[test]
    fn test_inspect_attr() {
        let style = DEFAULT_STYLES.get("inspect.attr").unwrap();
        assert_eq!(style.italic(), Some(true));
        assert_eq!(style.color().unwrap().name, "yellow");
    }

    #[test]
    fn test_inspect_attr_dunder() {
        let style = DEFAULT_STYLES.get("inspect.attr.dunder").unwrap();
        assert_eq!(style.italic(), Some(true));
        assert_eq!(style.dim(), Some(true));
        assert_eq!(style.color().unwrap().name, "yellow");
    }

    #[test]
    fn test_logging_level_critical() {
        let style = DEFAULT_STYLES.get("logging.level.critical").unwrap();
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.reverse(), Some(true));
        assert_eq!(style.color().unwrap().name, "red");
    }

    #[test]
    fn test_repr_str() {
        let style = DEFAULT_STYLES.get("repr.str").unwrap();
        assert_eq!(style.italic(), Some(false));
        assert_eq!(style.bold(), Some(false));
        assert_eq!(style.color().unwrap().name, "green");
    }

    #[test]
    fn test_bar_complete() {
        let style = DEFAULT_STYLES.get("bar.complete").unwrap();
        assert!(style.color().is_some());
    }

    #[test]
    fn test_bar_back() {
        let style = DEFAULT_STYLES.get("bar.back").unwrap();
        assert!(style.color().is_some());
    }

    #[test]
    fn test_markdown_code() {
        let style = DEFAULT_STYLES.get("markdown.code").unwrap();
        assert_eq!(style.bold(), Some(true));
        assert_eq!(style.color().unwrap().name, "cyan");
        assert_eq!(style.bgcolor().unwrap().name, "black");
    }

    #[test]
    fn test_null_styles() {
        let null_names = [
            "none",
            "inspect.equals",
            "log.level",
            "log.message",
            "rule.text",
            "prompt",
            "pretty",
            "table.cell",
            "traceback.text",
            "traceback.exc_value",
            "progress.description",
            "tree",
            "tree.line",
            "markdown.paragraph",
            "markdown.text",
            "markdown.item",
            "markdown.h1.border",
        ];
        for name in &null_names {
            let style = DEFAULT_STYLES.get(*name).unwrap_or_else(|| {
                panic!("Missing null style: {}", name);
            });
            assert!(style.is_null(), "Expected '{}' to be null style", name);
        }
    }

    #[test]
    fn test_default_styles_clone() {
        let styles = default_styles();
        assert_eq!(styles.len(), DEFAULT_STYLES.len());
    }

    #[test]
    fn test_all_known_keys_present() {
        let expected_keys = vec![
            "none",
            "reset",
            "dim",
            "bright",
            "bold",
            "strong",
            "code",
            "italic",
            "emphasize",
            "underline",
            "blink",
            "blink2",
            "reverse",
            "strike",
            "black",
            "red",
            "green",
            "yellow",
            "magenta",
            "cyan",
            "white",
            "inspect.attr",
            "inspect.attr.dunder",
            "inspect.callable",
            "inspect.async_def",
            "inspect.def",
            "inspect.class",
            "inspect.error",
            "inspect.equals",
            "inspect.help",
            "inspect.doc",
            "inspect.value.border",
            "live.ellipsis",
            "layout.tree.row",
            "layout.tree.column",
            "logging.keyword",
            "logging.level.notset",
            "logging.level.debug",
            "logging.level.info",
            "logging.level.warning",
            "logging.level.error",
            "logging.level.critical",
            "log.level",
            "log.time",
            "log.message",
            "log.path",
            "repr.ellipsis",
            "repr.indent",
            "repr.error",
            "repr.str",
            "repr.brace",
            "repr.comma",
            "repr.ipv4",
            "repr.ipv6",
            "repr.eui48",
            "repr.eui64",
            "repr.tag_start",
            "repr.tag_name",
            "repr.tag_contents",
            "repr.tag_end",
            "repr.attrib_name",
            "repr.attrib_equal",
            "repr.attrib_value",
            "repr.number",
            "repr.number_complex",
            "repr.bool_true",
            "repr.bool_false",
            "repr.none",
            "repr.url",
            "repr.uuid",
            "repr.call",
            "repr.path",
            "repr.filename",
            "rule.line",
            "rule.text",
            "json.brace",
            "json.bool_true",
            "json.bool_false",
            "json.null",
            "json.number",
            "json.str",
            "json.key",
            "prompt",
            "prompt.choices",
            "prompt.default",
            "prompt.invalid",
            "prompt.invalid.choice",
            "pretty",
            "scope.border",
            "scope.key",
            "scope.key.special",
            "scope.equals",
            "table.header",
            "table.footer",
            "table.cell",
            "table.title",
            "table.caption",
            "traceback.error",
            "traceback.border.syntax_error",
            "traceback.border",
            "traceback.text",
            "traceback.title",
            "traceback.exc_type",
            "traceback.exc_value",
            "traceback.offset",
            "traceback.error_range",
            "traceback.note",
            "traceback.group.border",
            "bar.back",
            "bar.complete",
            "bar.finished",
            "bar.pulse",
            "progress.description",
            "progress.filesize",
            "progress.filesize.total",
            "progress.download",
            "progress.elapsed",
            "progress.percentage",
            "progress.remaining",
            "progress.data.speed",
            "progress.spinner",
            "status.spinner",
            "tree",
            "tree.line",
            "markdown.paragraph",
            "markdown.text",
            "markdown.em",
            "markdown.emph",
            "markdown.strong",
            "markdown.code",
            "markdown.code_block",
            "markdown.block_quote",
            "markdown.list",
            "markdown.item",
            "markdown.item.bullet",
            "markdown.item.number",
            "markdown.hr",
            "markdown.h1.border",
            "markdown.h1",
            "markdown.h2",
            "markdown.h3",
            "markdown.h4",
            "markdown.h5",
            "markdown.h6",
            "markdown.h7",
            "markdown.link",
            "markdown.link_url",
            "markdown.s",
            "markdown.table.border",
            "markdown.table.header",
            "iso8601.date",
            "iso8601.time",
            "iso8601.timezone",
        ];
        for key in &expected_keys {
            assert!(
                DEFAULT_STYLES.contains_key(*key),
                "Missing default style: {}",
                key
            );
        }
        assert_eq!(expected_keys.len(), 153);
    }
}
