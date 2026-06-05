//! Compile-time validator and expander for gilt markup strings.
//!
//! The `text!` macro validates a gilt markup literal at compile time — catching
//! unclosed brackets, mismatched/unclosed tags, and unknown style tokens —
//! then expands to a `::gilt::text::Text`.
//!
//! # Limitations
//!
//! `text!` accepts only **string literals**. For dynamic markup built at
//! runtime, use [`gilt::text::Text::from_markup`] directly.
//!
//! # Vocabulary
//!
//! The validator mirrors the **exact** vocabulary accepted by
//! `gilt::style::Style::parse_strict` / `gilt::color::Color::parse`:
//!
//! - **Attributes** (case-insensitive): `bold`/`b`, `dim`/`d`, `italic`/`i`,
//!   `underline`/`u`, `blink`, `blink2`, `reverse`/`r`, `conceal`/`c`,
//!   `strike`/`s`, `underline2`/`uu`, `frame`, `encircle`, `overline`/`o`
//! - **`not <attr>`**: negates any attribute above
//! - **`on <color>`**: background color
//! - **`link <url>`** or **`link=<url>`**: hyperlink
//! - **Named colors**: the 16 standard ANSI names (`black`, `red`, …,
//!   `bright_white`) plus the full extended 256-color set mirrored from
//!   `src/color/mod.rs` (also their `gray`/`grey` spelling variants)
//! - **`default`**: terminal default color
//! - **`#rrggbb`**: 6-hex-digit truecolor
//! - **`color(N)`**: 0–255 ANSI palette index
//! - **`rgb(R,G,B)`**: RGB truecolor
//! - **Underline styles**: `single`, `double`, `curly`, `dotted`, `dashed`
//! - **`underline_color(<color>)`**: underline color token
//! - **`[@key=val]` / `[@flag]`**: meta tags — always valid, never validated
//!   as style tokens

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Validate gilt markup at compile time and expand to a `::gilt::text::Text`.
///
/// Accepts a single string literal.  Typos that would be silent at runtime
/// become `cargo build` errors.
pub fn text_macro_impl(lit: LitStr) -> TokenStream {
    let value = lit.value();

    if let Err(msg) = validate_markup(&value) {
        return syn::Error::new_spanned(&lit, msg).to_compile_error();
    }

    quote! {
        ::gilt::text::Text::from_markup(#lit)
            .expect("text! markup validated at compile time")
    }
}

// ---------------------------------------------------------------------------
// Markup validator
// ---------------------------------------------------------------------------

/// Validate gilt markup, returning `Ok(())` or `Err(human-readable message)`.
///
/// The regex used by `markup::RE_MARKUP` is:
///   `(\\*)(\[[a-z#/@][^\[]*?\])`
/// so only tags whose first character matches `[a-z#/@]` are parsed; everything
/// else is plain text.
fn validate_markup(markup: &str) -> Result<(), String> {
    // Fast path: no `[` means no tags.
    if !markup.contains('[') {
        return Ok(());
    }

    // Stack of open tag names (already lowercased/trimmed).
    let mut tag_stack: Vec<String> = Vec::new();

    let mut chars = markup.char_indices().peekable();

    while let Some((pos, ch)) = chars.next() {
        if ch == '\\' {
            // One backslash: consume the next char (escape sequence), whatever it is.
            chars.next();
            continue;
        }

        if ch != '[' {
            continue;
        }

        // We have an unescaped `[`.  Collect until matching `]`.
        // The runtime regex requires NO nested `[` inside a tag (char class
        // `[^\[]*?`), so a `[` before `]` means this bracket is not a tag.
        let mut inner = String::new();
        let mut closed = false;
        let mut had_open_bracket = false;

        loop {
            match chars.next() {
                None => break,
                Some((_, ']')) => {
                    closed = true;
                    break;
                }
                Some((_, '[')) => {
                    // Nested `[` — the runtime regex would not match this as a tag.
                    had_open_bracket = true;
                    inner.push('[');
                }
                Some((_, c)) => {
                    inner.push(c);
                }
            }
        }

        if !closed {
            return Err(format!(
                "unclosed `[` at byte offset {pos}: no matching `]` found"
            ));
        }

        if had_open_bracket {
            // Runtime regex doesn't treat this as a tag — skip.
            continue;
        }

        // Check the first character matches `[a-z#/@]` (runtime regex requirement).
        let first = inner.chars().next().unwrap_or('\0');
        if !matches!(first, 'a'..='z' | '#' | '/' | '@') {
            // Not a tag — plain text according to the runtime parser.
            continue;
        }

        // Handle `=` — split name from parameters.
        // e.g. `[link=https://example.com]` → name="link", params=Some("https://example.com")
        // The runtime Tag::to_string() formats this as "link https://example.com",
        // and Style::parse_strict receives that combined form.
        let (tag_name_raw, params) = if let Some(eq_pos) = inner.find('=') {
            (&inner[..eq_pos], Some(&inner[eq_pos + 1..]))
        } else {
            (inner.as_str(), None)
        };
        let tag_name = tag_name_raw.trim().to_lowercase();

        if tag_name.starts_with('@') {
            // Meta tag — always valid; push onto stack.
            tag_stack.push(tag_name);
            continue;
        }

        if let Some(close_suffix) = tag_name.strip_prefix('/') {
            // Closing tag.
            let close_name = close_suffix.trim().to_string();

            if close_name.is_empty() {
                // `[/]` — implicit close: pop the most recent tag.
                if tag_stack.pop().is_none() {
                    return Err(format!(
                        "closing tag `[/]` at byte offset {pos} with no open tag to close"
                    ));
                }
            } else {
                // Explicit close `[/name]` — find the matching open tag anywhere
                // in the stack (gilt uses `rposition`, so we match the deepest
                // matching tag, not just the top).
                let found = tag_stack
                    .iter()
                    .rposition(|t| t.trim_start_matches('@') == close_name);
                match found {
                    Some(idx) => {
                        tag_stack.remove(idx);
                    }
                    None => {
                        return Err(format!(
                            "closing tag `[/{close_name}]` at byte offset {pos} \
                             does not match any open tag (open stack: [{}])",
                            tag_stack.join(", ")
                        ));
                    }
                }
            }
        } else {
            // Opening style tag — build the spec string the same way as
            // Tag::to_string() does at runtime: "name" or "name params".
            // This ensures [link=url] becomes "link url" which parse_internal
            // handles as `link <url>` correctly.
            let style_spec = match params {
                Some(p) => format!("{tag_name} {p}"),
                None => tag_name.clone(),
            };
            validate_style_spec(&style_spec, pos)?;
            tag_stack.push(tag_name);
        }
    }

    // Unclosed tags are **valid** in gilt (they apply to the rest of the text).
    // Do NOT error on a non-empty stack at end of input.
    Ok(())
}

// ---------------------------------------------------------------------------
// Style token validator
// ---------------------------------------------------------------------------

/// Validate the tokens in a style tag specification (e.g. `"bold red"`,
/// `"on blue"`, `"not italic"`).
///
/// Mirrors `Style::parse_internal` exactly.
fn validate_style_spec(spec: &str, tag_pos: usize) -> Result<(), String> {
    let words: Vec<&str> = spec.split_whitespace().collect();
    let mut i = 0;

    while i < words.len() {
        let word = words[i].to_lowercase();

        match word.as_str() {
            "on" => {
                i += 1;
                if i >= words.len() {
                    return Err(format!(
                        "style tag `[{spec}]` at byte offset {tag_pos}: \
                         expected color after `on`"
                    ));
                }
                let color_word = words[i].to_lowercase();
                validate_color_token(&color_word, spec, tag_pos)?;
            }
            "not" => {
                i += 1;
                if i >= words.len() {
                    return Err(format!(
                        "style tag `[{spec}]` at byte offset {tag_pos}: \
                         expected attribute after `not`"
                    ));
                }
                let attr = words[i].to_lowercase();
                if !is_attribute_name(&attr) {
                    return Err(format!(
                        "style tag `[{spec}]` at byte offset {tag_pos}: \
                         unknown attribute `{attr}` after `not` \
                         (known attributes: bold/b, dim/d, italic/i, underline/u, \
                         blink, blink2, reverse/r, conceal/c, strike/s, \
                         underline2/uu, frame, encircle, overline/o)"
                    ));
                }
            }
            "link" => {
                // `link <url>` — consume the URL token.
                i += 1;
                if i >= words.len() {
                    return Err(format!(
                        "style tag `[{spec}]` at byte offset {tag_pos}: \
                         expected URL after `link`"
                    ));
                }
                // URL is any non-empty string — accepted as-is.
            }
            _ => {
                // `link=URL` inline form.
                if let Some(url) = word.strip_prefix("link=") {
                    if url.is_empty() {
                        return Err(format!(
                            "style tag `[{spec}]` at byte offset {tag_pos}: \
                             expected URL after `link=`"
                        ));
                    }
                } else if is_underline_style_name(&word) {
                    // Underline style — accepted.
                } else if word.starts_with("underline_color(") && word.ends_with(')') {
                    let inner = &word["underline_color(".len()..word.len() - 1];
                    validate_color_token(inner, spec, tag_pos)?;
                } else if is_attribute_name(&word) {
                    // Attribute — accepted.
                } else {
                    // Try as foreground color.
                    validate_color_token(&word, spec, tag_pos)?;
                }
            }
        }

        i += 1;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Color validator
// ---------------------------------------------------------------------------

fn validate_color_token(color: &str, spec: &str, tag_pos: usize) -> Result<(), String> {
    // "default"
    if color == "default" {
        return Ok(());
    }

    // "#rrggbb"
    if let Some(hex) = color.strip_prefix('#') {
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(());
        }
        return Err(format!(
            "style tag `[{spec}]` at byte offset {tag_pos}: \
             invalid hex color `#{hex}` — must be exactly 6 hex digits (e.g. `#ff0000`)"
        ));
    }

    // "color(N)"
    if color.starts_with("color(") && color.ends_with(')') {
        let n_str = &color["color(".len()..color.len() - 1];
        match n_str.parse::<u16>() {
            Ok(n) if n <= 255 => return Ok(()),
            _ => {
                return Err(format!(
                    "style tag `[{spec}]` at byte offset {tag_pos}: \
                     invalid color index in `{color}` — must be 0–255"
                ));
            }
        }
    }

    // "rgb(R,G,B)"
    if color.starts_with("rgb(") && color.ends_with(')') {
        let inner = &color["rgb(".len()..color.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 && parts.iter().all(|p| p.trim().parse::<u8>().is_ok()) {
            return Ok(());
        }
        return Err(format!(
            "style tag `[{spec}]` at byte offset {tag_pos}: \
             invalid rgb color `{color}` — must be `rgb(R,G,B)` with R,G,B in 0–255"
        ));
    }

    // Named color.
    if is_named_color(color) {
        return Ok(());
    }

    Err(format!(
        "style tag `[{spec}]` at byte offset {tag_pos}: \
         unknown style token `{color}` \
         (expected a color name, `#rrggbb`, `color(N)`, `rgb(R,G,B)`, \
         `on <color>`, `not <attr>`, `link <url>`, or an attribute like `bold`)"
    ))
}

// ---------------------------------------------------------------------------
// Attribute names
// ---------------------------------------------------------------------------

fn is_attribute_name(name: &str) -> bool {
    matches!(
        name,
        "bold"
            | "b"
            | "dim"
            | "d"
            | "italic"
            | "i"
            | "underline"
            | "u"
            | "blink"
            | "blink2"
            | "reverse"
            | "r"
            | "conceal"
            | "c"
            | "strike"
            | "s"
            | "underline2"
            | "uu"
            | "frame"
            | "encircle"
            | "overline"
            | "o"
    )
}

// ---------------------------------------------------------------------------
// Underline style names
// ---------------------------------------------------------------------------

fn is_underline_style_name(name: &str) -> bool {
    matches!(name, "single" | "double" | "curly" | "dotted" | "dashed")
}

// ---------------------------------------------------------------------------
// Named color vocabulary — mirrors `get_ansi_color_number` in color/mod.rs
// ---------------------------------------------------------------------------

fn is_named_color(name: &str) -> bool {
    matches!(
        name,
        // Standard 16 ANSI colors
        "black"
            | "red"
            | "green"
            | "yellow"
            | "blue"
            | "magenta"
            | "cyan"
            | "white"
            | "bright_black"
            | "bright_red"
            | "bright_green"
            | "bright_yellow"
            | "bright_blue"
            | "bright_magenta"
            | "bright_cyan"
            | "bright_white"
            // Extended 256-color names (both grey/gray spellings)
            | "grey0"
            | "gray0"
            | "navy_blue"
            | "dark_blue"
            | "blue3"
            | "blue1"
            | "dark_green"
            | "deep_sky_blue4"
            | "dodger_blue3"
            | "dodger_blue2"
            | "green4"
            | "spring_green4"
            | "turquoise4"
            | "deep_sky_blue3"
            | "dodger_blue1"
            | "green3"
            | "spring_green3"
            | "dark_cyan"
            | "light_sea_green"
            | "deep_sky_blue2"
            | "deep_sky_blue1"
            | "spring_green2"
            | "cyan3"
            | "dark_turquoise"
            | "turquoise2"
            | "green1"
            | "spring_green1"
            | "medium_spring_green"
            | "cyan2"
            | "cyan1"
            | "dark_red"
            | "deep_pink4"
            | "purple4"
            | "purple3"
            | "blue_violet"
            | "orange4"
            | "grey37"
            | "gray37"
            | "medium_purple4"
            | "slate_blue3"
            | "royal_blue1"
            | "chartreuse4"
            | "dark_sea_green4"
            | "pale_turquoise4"
            | "steel_blue"
            | "steel_blue3"
            | "cornflower_blue"
            | "chartreuse3"
            | "cadet_blue"
            | "sky_blue3"
            | "steel_blue1"
            | "pale_green3"
            | "sea_green3"
            | "aquamarine3"
            | "medium_turquoise"
            | "chartreuse2"
            | "sea_green2"
            | "sea_green1"
            | "aquamarine1"
            | "dark_slate_gray2"
            | "dark_magenta"
            | "dark_violet"
            | "purple"
            | "light_pink4"
            | "plum4"
            | "medium_purple3"
            | "slate_blue1"
            | "yellow4"
            | "wheat4"
            | "grey53"
            | "gray53"
            | "light_slate_grey"
            | "light_slate_gray"
            | "medium_purple"
            | "light_slate_blue"
            | "dark_olive_green3"
            | "dark_sea_green"
            | "light_sky_blue3"
            | "sky_blue2"
            | "dark_sea_green3"
            | "dark_slate_gray3"
            | "sky_blue1"
            | "chartreuse1"
            | "light_green"
            | "pale_green1"
            | "dark_slate_gray1"
            | "red3"
            | "medium_violet_red"
            | "magenta3"
            | "dark_orange3"
            | "indian_red"
            | "hot_pink3"
            | "medium_orchid3"
            | "medium_orchid"
            | "medium_purple2"
            | "dark_goldenrod"
            | "light_salmon3"
            | "rosy_brown"
            | "grey63"
            | "gray63"
            | "medium_purple1"
            | "gold3"
            | "dark_khaki"
            | "navajo_white3"
            | "grey69"
            | "gray69"
            | "light_steel_blue3"
            | "light_steel_blue"
            | "yellow3"
            | "dark_sea_green2"
            | "light_cyan3"
            | "light_sky_blue1"
            | "green_yellow"
            | "dark_olive_green2"
            | "dark_sea_green1"
            | "pale_turquoise1"
            | "deep_pink3"
            | "magenta2"
            | "hot_pink2"
            | "orchid"
            | "medium_orchid1"
            | "orange3"
            | "light_pink3"
            | "pink3"
            | "plum3"
            | "violet"
            | "light_goldenrod3"
            | "tan"
            | "misty_rose3"
            | "thistle3"
            | "plum2"
            | "khaki3"
            | "light_goldenrod2"
            | "light_yellow3"
            | "grey84"
            | "gray84"
            | "light_steel_blue1"
            | "yellow2"
            | "dark_olive_green1"
            | "honeydew2"
            | "light_cyan1"
            | "red1"
            | "deep_pink2"
            | "deep_pink1"
            | "magenta1"
            | "orange_red1"
            | "indian_red1"
            | "hot_pink"
            | "dark_orange"
            | "salmon1"
            | "light_coral"
            | "pale_violet_red1"
            | "orchid2"
            | "orchid1"
            | "orange1"
            | "sandy_brown"
            | "light_salmon1"
            | "light_pink1"
            | "pink1"
            | "plum1"
            | "gold1"
            | "navajo_white1"
            | "misty_rose1"
            | "thistle1"
            | "yellow1"
            | "light_goldenrod1"
            | "khaki1"
            | "wheat1"
            | "cornsilk1"
            | "grey100"
            | "gray100"
            | "grey3"
            | "gray3"
            | "grey7"
            | "gray7"
            | "grey11"
            | "gray11"
            | "grey15"
            | "gray15"
            | "grey19"
            | "gray19"
            | "grey23"
            | "gray23"
            | "grey27"
            | "gray27"
            | "grey30"
            | "gray30"
            | "grey35"
            | "gray35"
            | "grey39"
            | "gray39"
            | "grey42"
            | "gray42"
            | "grey46"
            | "gray46"
            | "grey50"
            | "gray50"
            | "grey54"
            | "gray54"
            | "grey58"
            | "gray58"
            | "grey62"
            | "gray62"
            | "grey66"
            | "gray66"
            | "grey70"
            | "gray70"
            | "grey74"
            | "gray74"
            | "grey78"
            | "gray78"
            | "grey82"
            | "gray82"
            | "grey85"
            | "gray85"
            | "grey89"
            | "gray89"
            | "grey93"
            | "gray93"
    )
}

// ---------------------------------------------------------------------------
// Unit tests for the validator (no proc-macro machinery needed)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- validate_markup pass cases ----

    #[test]
    fn test_valid_simple_bold() {
        assert!(validate_markup("[bold]hello[/bold]").is_ok());
    }

    #[test]
    fn test_valid_bold_red() {
        assert!(validate_markup("[bold red]Error:[/] file not found").is_ok());
    }

    #[test]
    fn test_valid_implicit_close() {
        assert!(validate_markup("[italic]text[/]").is_ok());
    }

    #[test]
    fn test_valid_unclosed_tag() {
        // gilt treats unclosed tags as valid — applies to rest of text.
        assert!(validate_markup("[bold]hello").is_ok());
    }

    #[test]
    fn test_valid_nested() {
        assert!(validate_markup("[green]X[blue]Y[/blue]Z[/green]").is_ok());
    }

    #[test]
    fn test_valid_overlap() {
        // gilt supports overlapping tags (close out-of-order is OK because
        // it searches the full stack, not just the top).
        assert!(validate_markup("[green]X[bold]Y[/green]Z[/bold]").is_ok());
    }

    #[test]
    fn test_valid_hex_color() {
        assert!(validate_markup("[#ff0000]red text[/]").is_ok());
    }

    #[test]
    fn test_valid_color_n() {
        assert!(validate_markup("[color(42)]text[/]").is_ok());
    }

    #[test]
    fn test_valid_rgb() {
        assert!(validate_markup("[rgb(255,0,128)]text[/]").is_ok());
    }

    #[test]
    fn test_valid_on_color() {
        assert!(validate_markup("[on blue]text[/]").is_ok());
    }

    #[test]
    fn test_valid_not_attr() {
        assert!(validate_markup("[not bold]text[/]").is_ok());
    }

    #[test]
    fn test_valid_link_space() {
        assert!(validate_markup("[link https://example.com]click[/]").is_ok());
    }

    #[test]
    fn test_valid_link_eq() {
        assert!(validate_markup("[link=https://example.com]click[/link]").is_ok());
    }

    #[test]
    fn test_valid_meta_tag_flag() {
        assert!(validate_markup("[@click]hello[/]").is_ok());
    }

    #[test]
    fn test_valid_meta_tag_kv() {
        assert!(validate_markup("[@key=val]text[/]").is_ok());
    }

    #[test]
    fn test_valid_underline_styles() {
        for s in &["single", "double", "curly", "dotted", "dashed"] {
            assert!(
                validate_markup(&format!("[{s}]text[/]")).is_ok(),
                "underline style {s} must be valid"
            );
        }
    }

    #[test]
    fn test_valid_underline_color() {
        assert!(validate_markup("[underline_color(red)]text[/]").is_ok());
    }

    #[test]
    fn test_valid_empty() {
        assert!(validate_markup("").is_ok());
    }

    #[test]
    fn test_valid_no_tags() {
        assert!(validate_markup("plain text here").is_ok());
    }

    #[test]
    fn test_valid_escaped_bracket() {
        // `\[bold]` — the tag is escaped, treated as literal text.
        assert!(validate_markup(r"\[bold]hello").is_ok());
    }

    #[test]
    fn test_valid_default_color() {
        assert!(validate_markup("[default]text[/]").is_ok());
    }

    // Anti-false-positive: many varied valid strings must all pass.
    #[test]
    fn test_anti_false_positive() {
        let valid = vec![
            "[bold]text[/bold]",
            "[bold red]text[/]",
            "[italic]x[/italic]",
            "[underline]y[/underline]",
            "[on green]bg[/]",
            "[bright_blue]hi[/]",
            "[#aabbcc]hex[/]",
            "[color(200)]indexed[/]",
            "[rgb(10,20,30)]rgb[/]",
            "[not bold]normal[/]",
            "[link https://example.com]url[/]",
            "[@meta=val]tagged[/]",
            "[bold italic red]combined[/]",
            "[green]X[blue]Y[/blue]Z[/green]",
            "[bold]outer[italic]inner[/italic][/bold]",
            "no tags at all",
            "",
            r"\[escaped]not a tag",
            "[dim]faded[/dim]",
            "[strike]struck[/strike]",
            "[reverse]flipped[/reverse]",
            "[blink]blinky[/blink]",
            "[on bright_red]error bg[/]",
            "[cyan]text[/cyan]",
            "[magenta]m[/magenta]",
        ];
        for markup in &valid {
            assert!(
                validate_markup(markup).is_ok(),
                "FAIL: valid markup `{markup}` was rejected: {:?}",
                validate_markup(markup)
            );
        }
    }

    // ---- validate_markup fail cases ----

    #[test]
    fn test_error_unclosed_bracket() {
        assert!(validate_markup("[bold").is_err());
    }

    #[test]
    fn test_error_mismatched_explicit_close() {
        // `[bold]x[/italic]` — "italic" not on stack.
        let r = validate_markup("[bold]x[/italic]");
        assert!(r.is_err(), "mismatched explicit close must fail");
    }

    #[test]
    fn test_error_close_with_nothing_open() {
        // `[/]` with empty stack.
        assert!(validate_markup("[/]").is_err());
    }

    #[test]
    fn test_error_explicit_close_empty_stack() {
        // `[/bold]` with empty stack.
        assert!(validate_markup("[/bold]").is_err());
    }

    #[test]
    fn test_error_unknown_style_token() {
        let r = validate_markup("[blod]text[/]");
        assert!(r.is_err(), "typo 'blod' must be caught");
        let msg = r.unwrap_err();
        assert!(
            msg.contains("blod"),
            "error message must name the bad token: {msg}"
        );
    }

    #[test]
    fn test_error_unknown_color() {
        let r = validate_markup("[purpel]text[/]");
        assert!(r.is_err());
    }

    #[test]
    fn test_error_on_without_color() {
        assert!(validate_markup("[on]text[/]").is_err());
    }

    #[test]
    fn test_error_not_without_attr() {
        assert!(validate_markup("[not]text[/]").is_err());
    }

    #[test]
    fn test_error_not_unknown_attr() {
        assert!(validate_markup("[not invisible]text[/]").is_err());
    }

    #[test]
    fn test_error_link_without_url() {
        assert!(validate_markup("[link]text[/]").is_err());
    }

    #[test]
    fn test_error_bad_hex_length() {
        assert!(validate_markup("[#f00]text[/]").is_err());
    }

    #[test]
    fn test_error_bad_color_n_overflow() {
        assert!(validate_markup("[color(256)]text[/]").is_err());
    }

    #[test]
    fn test_error_bad_rgb() {
        assert!(validate_markup("[rgb(300,0,0)]text[/]").is_err());
    }
}
