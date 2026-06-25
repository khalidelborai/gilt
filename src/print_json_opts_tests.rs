//! Tests for the crate-level `print_json_opts` free function (Plan 7.24 Task 2).
//!
//! Verifies the additive `print_json_opts(json, data, ensure_ascii)`:
//! - `data=Some` serializes the value (instead of parsing `json`).
//! - `ensure_ascii=true` produces `\uXXXX` escapes for non-ASCII.
//! - `ensure_ascii=false` keeps non-ASCII characters as-is.

use serde_json::json;

#[test]
fn print_json_opts_data_with_ensure_ascii_true_escapes_non_ascii() {
    // ensure_ascii=true → "é" is escaped to "\u00e9".
    let v = json!({ "k": "é" });
    let out =
        crate::format_json_for_test(None, Some(&v), true).expect("serialization should succeed");
    assert!(
        out.contains(r"\u00e9"),
        "ensure_ascii=true must escape non-ASCII; got: {out:?}"
    );
    assert!(
        !out.contains('é'),
        "ensure_ascii=true must not emit raw non-ASCII; got: {out:?}"
    );
}

#[test]
fn print_json_opts_data_with_ensure_ascii_false_keeps_non_ascii() {
    // ensure_ascii=false → "é" stays as the raw character.
    let v = json!({ "k": "é" });
    let out =
        crate::format_json_for_test(None, Some(&v), false).expect("serialization should succeed");
    assert!(
        out.contains('é'),
        "ensure_ascii=false must keep non-ASCII; got: {out:?}"
    );
    assert!(
        !out.contains(r"\u00e9"),
        "ensure_ascii=false must NOT escape non-ASCII; got: {out:?}"
    );
}

#[test]
fn print_json_opts_json_string_path_routes_through_serializer() {
    // When `data` is None, the provided JSON string is parsed and
    // re-serialized. ASCII content should pass through unchanged.
    let out = crate::format_json_for_test(Some(r#"{"k": "v"}"#), None, true)
        .expect("serialization should succeed");
    assert!(
        out.contains(r#""k""#),
        "key must be preserved; got: {out:?}"
    );
    assert!(
        out.contains(r#""v""#),
        "value must be preserved; got: {out:?}"
    );
}

#[test]
fn print_json_opts_data_takes_precedence_over_json() {
    // When both are provided, `data` wins (rich parity).
    let v = json!({ "from_data": 1 });
    let out = crate::format_json_for_test(Some(r#"{"from_json": 0}"#), Some(&v), true)
        .expect("serialization should succeed");
    assert!(
        out.contains("from_data"),
        "data must win over json; got: {out:?}"
    );
    assert!(
        !out.contains("from_json"),
        "json string must be ignored when data is Some; got: {out:?}"
    );
}
