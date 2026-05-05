//! Unit tests + insta snapshot tests for all seven derives.
//!
//! Split from `lib.rs` for readability; v0.11.4 CHANGELOG flagged this as
//! planned cleanup. Tests still run via `cargo test -p gilt-derive --lib`
//! because `lib.rs` declares `#[cfg(test)] mod tests;`.

use super::*;
use crate::shared::{box_style_tokens, justify_tokens, snake_to_title_case};
use syn::LitStr;

// -- snake_to_title_case -----------------------------------------------

#[test]
fn test_snake_to_title_case() {
    assert_eq!(snake_to_title_case("first_name"), "First Name");
    assert_eq!(snake_to_title_case("age"), "Age");
    assert_eq!(snake_to_title_case("department_id"), "Department Id");
    assert_eq!(snake_to_title_case("a_b_c"), "A B C");
    assert_eq!(snake_to_title_case("single"), "Single");
}

#[test]
fn test_snake_to_title_case_edge_cases() {
    assert_eq!(snake_to_title_case(""), "");
    assert_eq!(snake_to_title_case("_leading"), "Leading");
    assert_eq!(snake_to_title_case("trailing_"), "Trailing");
    assert_eq!(snake_to_title_case("__double__"), "Double");
    assert_eq!(snake_to_title_case("ALL_CAPS"), "ALL CAPS");
}

// -- box_style_tokens --------------------------------------------------

#[test]
fn test_box_style_tokens_valid() {
    let valid = [
        "ASCII",
        "ASCII2",
        "ASCII_DOUBLE_HEAD",
        "SQUARE",
        "SQUARE_DOUBLE_HEAD",
        "MINIMAL",
        "MINIMAL_HEAVY_HEAD",
        "MINIMAL_DOUBLE_HEAD",
        "SIMPLE",
        "SIMPLE_HEAD",
        "SIMPLE_HEAVY",
        "HORIZONTALS",
        "ROUNDED",
        "HEAVY",
        "HEAVY_EDGE",
        "HEAVY_HEAD",
        "DOUBLE",
        "DOUBLE_EDGE",
        "MARKDOWN",
    ];
    for name in valid {
        let lit = LitStr::new(name, Span::call_site());
        assert!(
            box_style_tokens(&lit).is_ok(),
            "box_style_tokens should accept `{}`",
            name
        );
    }
}

#[test]
fn test_box_style_tokens_invalid() {
    let lit = LitStr::new("NONEXISTENT", Span::call_site());
    let result = box_style_tokens(&lit);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("unknown box_style"),
        "error should mention unknown box_style, got: {}",
        err_msg
    );
}

// -- justify_tokens ----------------------------------------------------

#[test]
fn test_justify_tokens_valid() {
    for name in ["left", "center", "right", "full"] {
        let lit = LitStr::new(name, Span::call_site());
        assert!(
            justify_tokens(&lit).is_ok(),
            "justify_tokens should accept `{}`",
            name
        );
    }
}

#[test]
fn test_justify_tokens_invalid() {
    let lit = LitStr::new("middle", Span::call_site());
    let result = justify_tokens(&lit);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("unknown justify"),
        "error should mention unknown justify, got: {}",
        err_msg
    );
}

// -- table::TableAttr parsing -------------------------------------------------

#[test]
fn test_parse_table_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Title" };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "title");
    match attr.value {
        table::TableAttrValue::Str(s) => assert_eq!(s.value(), "My Title"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_table_attr_bool() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = true };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "expand");
    match attr.value {
        table::TableAttrValue::Bool(b) => assert!(b.value),
        _ => panic!("expected Bool"),
    }
}

#[test]
fn test_parse_table_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "expand");
    matches!(attr.value, table::TableAttrValue::Flag);
}

// -- table::ColumnAttr parsing ------------------------------------------------

#[test]
fn test_parse_column_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { header = "Name" };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "header");
    match attr.value {
        table::ColumnAttrValue::Str(s) => assert_eq!(s.value(), "Name"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_column_attr_int() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = 42 };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "width");
    match attr.value {
        table::ColumnAttrValue::Int(i) => assert_eq!(i.base10_parse::<usize>().unwrap(), 42),
        _ => panic!("expected Int"),
    }
}

#[test]
fn test_parse_column_attr_bool() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { no_wrap = true };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "no_wrap");
    match attr.value {
        table::ColumnAttrValue::Bool(b) => assert!(b.value),
        _ => panic!("expected Bool"),
    }
}

#[test]
fn test_parse_column_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "skip");
    matches!(attr.value, table::ColumnAttrValue::Flag);
}

// -- expect helpers ----------------------------------------------------

#[test]
fn test_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    let result = table::expect_str(&attr, "title");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "hello");
}

#[test]
fn test_expect_str_wrong_type() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    let result = table::expect_str(&attr, "title");
    assert!(result.is_err());
}

#[test]
fn test_expect_bool_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = false };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    let result = table::expect_bool(&attr, "expand");
    assert!(result.is_ok());
    assert!(!result.unwrap().value);
}

#[test]
fn test_expect_bool_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
    let attr: table::TableAttr = syn::parse2(tokens).unwrap();
    let result = table::expect_bool(&attr, "expand");
    assert!(result.is_ok());
    assert!(result.unwrap().value);
}

#[test]
fn test_col_expect_int_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = 10 };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    let result = table::col_expect_int(&attr, "width");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().base10_parse::<usize>().unwrap(), 10);
}

#[test]
fn test_col_expect_int_wrong_type() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = "ten" };
    let attr: table::ColumnAttr = syn::parse2(tokens).unwrap();
    let result = table::col_expect_int(&attr, "width");
    assert!(result.is_err());
}

// -- Full derive round-trip (syn parse) --------------------------------

#[test]
fn test_derive_basic_struct() {
    let input: DeriveInput = syn::parse_quote! {
        struct Employee {
            name: String,
            age: u32,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(
        result.is_ok(),
        "derive_table_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_table"));
    assert!(tokens.contains("\"Name\""));
    assert!(tokens.contains("\"Age\""));
}

#[test]
fn test_derive_with_skip() {
    let input: DeriveInput = syn::parse_quote! {
        struct Data {
            visible: String,
            #[column(skip)]
            hidden: u64,
            also_visible: i32,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("\"Visible\""));
    assert!(tokens.contains("\"Also Visible\""));
    // The hidden field should not appear as a header.
    assert!(!tokens.contains("\"Hidden\""));
}

#[test]
fn test_derive_with_custom_header() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[column(header = "Full Name")]
            name: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("\"Full Name\""));
}

#[test]
fn test_derive_with_table_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(title = "My Table", box_style = "ROUNDED", show_lines = true)]
        struct Rec {
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("\"My Table\""));
    assert!(tokens.contains("ROUNDED"));
    assert!(tokens.contains("show_lines"));
}

#[test]
fn test_derive_with_column_justify() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[column(justify = "right")]
            amount: f64,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("JustifyMethod"));
    assert!(tokens.contains("Right"));
}

#[test]
fn test_derive_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

#[test]
fn test_derive_rejects_unknown_table_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(nonexistent = "value")]
        struct Rec {
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown table attribute"),);
}

#[test]
fn test_derive_rejects_unknown_column_attr() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[column(nonexistent = "value")]
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown column attribute"),);
}

#[test]
fn test_derive_rejects_invalid_justify() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[column(justify = "middle")]
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unknown justify"));
}

#[test]
fn test_derive_rejects_invalid_box_style() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(box_style = "FANCY")]
        struct Rec {
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown box_style"));
}

#[test]
fn test_derive_row_styles() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(row_styles = "bold, dim")]
        struct Rec {
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("row_styles"));
    assert!(tokens.contains("\"bold\""));
    assert!(tokens.contains("\"dim\""));
}

#[test]
fn test_derive_column_width_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[column(width = 20, min_width = 5, max_width = 50, ratio = 2)]
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("width"));
    assert!(tokens.contains("min_width"));
    assert!(tokens.contains("max_width"));
    assert!(tokens.contains("ratio"));
}

#[test]
fn test_derive_expand_flag() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(expand)]
        struct Rec {
            a: String,
        }
    };
    let result = table::derive_table_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("set_expand"));
}

// -- Panel derive tests ------------------------------------------------

#[test]
fn test_derive_panel_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct Server {
            name: String,
            cpu: f32,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(
        result.is_ok(),
        "derive_panel_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("to_panel"),
        "should generate to_panel method"
    );
    assert!(
        tokens.contains("\"Name\""),
        "should contain default label 'Name'"
    );
    assert!(
        tokens.contains("\"Cpu\""),
        "should contain default label 'Cpu'"
    );
    assert!(tokens.contains("Panel"), "should reference Panel type");
    assert!(
        tokens.contains("from_markup"),
        "should use from_markup for content"
    );
    // Default title should be the struct name.
    assert!(
        tokens.contains("\"Server\""),
        "default title should be struct name"
    );
}

#[test]
fn test_derive_panel_with_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        #[panel(
            title = "Server Status",
            subtitle = "Last updated",
            box_style = "HEAVY",
            border_style = "blue",
            style = "white",
            expand = false,
            highlight = true
        )]
        struct Server {
            #[field(label = "Host", style = "bold cyan")]
            name: String,
            cpu: f32,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(
        result.is_ok(),
        "derive_panel_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"Server Status\""),
        "should contain custom title"
    );
    assert!(
        tokens.contains("\"Last updated\""),
        "should contain subtitle"
    );
    assert!(tokens.contains("HEAVY"), "should reference HEAVY box style");
    assert!(tokens.contains("\"blue\""), "should contain border_style");
    assert!(tokens.contains("\"white\""), "should contain style");
    assert!(tokens.contains("expand"), "should set expand");
    assert!(tokens.contains("highlight"), "should set highlight");
    assert!(tokens.contains("\"Host\""), "should use custom label");
    assert!(
        tokens.contains("bold cyan"),
        "should contain field style markup"
    );
}

#[test]
fn test_derive_panel_skip_field() {
    let input: DeriveInput = syn::parse_quote! {
        struct Data {
            visible: String,
            #[field(skip)]
            hidden: u64,
            also_visible: i32,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"Visible\""),
        "should include visible field"
    );
    assert!(
        tokens.contains("\"Also Visible\""),
        "should include also_visible field"
    );
    // The hidden field should not appear.
    assert!(!tokens.contains("\"Hidden\""), "should skip hidden field");
    // Ensure the hidden field ident is not referenced.
    assert!(
        !tokens.contains("hidden"),
        "hidden field ident should not appear"
    );
}

#[test]
fn test_derive_panel_custom_labels() {
    let input: DeriveInput = syn::parse_quote! {
        struct Status {
            #[field(label = "Host Name")]
            server_name: String,
            #[field(label = "CPU %", style = "yellow")]
            cpu_usage: f32,
            #[field(label = "Mem (GB)")]
            memory_gb: f64,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"Host Name\""),
        "should use custom label 'Host Name'"
    );
    assert!(
        tokens.contains("\"CPU %\""),
        "should use custom label 'CPU %'"
    );
    assert!(
        tokens.contains("\"Mem (GB)\""),
        "should use custom label 'Mem (GB)'"
    );
    // Default Title Case labels should NOT appear.
    assert!(
        !tokens.contains("\"Server Name\""),
        "should not use default label"
    );
    assert!(
        !tokens.contains("\"Cpu Usage\""),
        "should not use default label"
    );
    assert!(
        !tokens.contains("\"Memory Gb\""),
        "should not use default label"
    );
}

#[test]
fn test_derive_panel_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

#[test]
fn test_derive_panel_rejects_unknown_panel_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[panel(nonexistent = "value")]
        struct Rec {
            a: String,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown panel attribute"),);
}

#[test]
fn test_derive_panel_rejects_unknown_field_attr() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[field(nonexistent = "value")]
            a: String,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown field attribute"),);
}

#[test]
fn test_derive_panel_rejects_invalid_box_style() {
    let input: DeriveInput = syn::parse_quote! {
        #[panel(box_style = "FANCY")]
        struct Rec {
            a: String,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown box_style"));
}

#[test]
fn test_derive_panel_title_style() {
    let input: DeriveInput = syn::parse_quote! {
        #[panel(title = "Info", title_style = "bold cyan")]
        struct Info {
            a: String,
        }
    };
    let result = panel::derive_panel_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("bold cyan"),
        "should apply title_style as markup"
    );
    assert!(tokens.contains("\"Info\""), "should contain title text");
}

// -- panel::PanelAttr parsing -------------------------------------------------

#[test]
fn test_parse_panel_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Panel" };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "title");
    match attr.value {
        panel::PanelAttrValue::Str(s) => assert_eq!(s.value(), "My Panel"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_panel_attr_bool() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = false };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "expand");
    match attr.value {
        panel::PanelAttrValue::Bool(b) => assert!(!b.value),
        _ => panic!("expected Bool"),
    }
}

#[test]
fn test_parse_panel_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { highlight };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "highlight");
    matches!(attr.value, panel::PanelAttrValue::Flag);
}

// -- panel::FieldAttr parsing -------------------------------------------------

#[test]
fn test_parse_field_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { label = "Host" };
    let attr: panel::FieldAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "label");
    match attr.value {
        panel::FieldAttrValue::Str(s) => assert_eq!(s.value(), "Host"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_field_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
    let attr: panel::FieldAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "skip");
    matches!(attr.value, panel::FieldAttrValue::Flag);
}

#[test]
fn test_panel_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    let result = panel::panel_expect_str(&attr, "title");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "hello");
}

#[test]
fn test_panel_expect_str_wrong_type() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    let result = panel::panel_expect_str(&attr, "title");
    assert!(result.is_err());
}

#[test]
fn test_panel_expect_bool_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
    let attr: panel::PanelAttr = syn::parse2(tokens).unwrap();
    let result = panel::panel_expect_bool(&attr, "expand");
    assert!(result.is_ok());
    assert!(result.unwrap().value);
}

#[test]
fn test_field_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { label = "Name" };
    let attr: panel::FieldAttr = syn::parse2(tokens).unwrap();
    let result = panel::field_expect_str(&attr, "label");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "Name");
}

#[test]
fn test_field_expect_bool_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
    let attr: panel::FieldAttr = syn::parse2(tokens).unwrap();
    let result = panel::field_expect_bool(&attr, "skip");
    assert!(result.is_ok());
    assert!(result.unwrap().value);
}

// -- Tree derive tests -------------------------------------------------

#[test]
fn test_derive_tree_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(
        result.is_ok(),
        "derive_tree_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_tree"), "should generate to_tree method");
    assert!(tokens.contains("Tree"), "should reference Tree type");
    assert!(tokens.contains("name"), "should use label field 'name'");
    assert!(
        tokens.contains("entries"),
        "should use children field 'entries'"
    );
    assert!(tokens.contains("children"), "should push to children vec");
}

#[test]
fn test_derive_tree_with_style() {
    let input: DeriveInput = syn::parse_quote! {
        #[tree(style = "bold", guide_style = "dim cyan")]
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(
        result.is_ok(),
        "derive_tree_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"bold\""),
        "should contain style string 'bold'"
    );
    assert!(
        tokens.contains("\"dim cyan\""),
        "should contain guide_style string 'dim cyan'"
    );
    assert!(
        tokens.contains("Style :: parse"),
        "should call Style::parse"
    );
}

#[test]
fn test_derive_tree_with_leaf() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
            #[tree(leaf)]
            size: u64,
            #[tree(leaf)]
            permissions: String,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(
        result.is_ok(),
        "derive_tree_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_tree"), "should generate to_tree method");
    assert!(
        tokens.contains("\"Size\""),
        "should contain leaf label 'Size'"
    );
    assert!(
        tokens.contains("\"Permissions\""),
        "should contain leaf label 'Permissions'"
    );
    assert!(
        tokens.contains("self . size"),
        "should reference leaf field 'size'"
    );
    assert!(
        tokens.contains("self . permissions"),
        "should reference leaf field 'permissions'"
    );
}

#[test]
fn test_derive_tree_missing_label() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(result.is_err(), "should error when no #[tree(label)] field");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("tree(label)"),
        "error should mention tree(label), got: {}",
        err_msg
    );
}

#[test]
fn test_derive_tree_missing_children() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(
        result.is_err(),
        "should error when no #[tree(children)] field"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("tree(children)"),
        "error should mention tree(children), got: {}",
        err_msg
    );
}

#[test]
fn test_derive_tree_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

#[test]
fn test_derive_tree_rejects_unknown_tree_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[tree(nonexistent = "value")]
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown tree attribute"),);
}

#[test]
fn test_derive_tree_rejects_unknown_field_role() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
            #[tree(bogus)]
            size: u64,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown tree field attribute"),);
}

#[test]
fn test_derive_tree_duplicate_label() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(label)]
            title: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("only one field"),);
}

#[test]
fn test_derive_tree_ignores_unannotated_fields() {
    let input: DeriveInput = syn::parse_quote! {
        struct FileEntry {
            #[tree(label)]
            name: String,
            #[tree(children)]
            entries: Vec<FileEntry>,
            ignored_field: u64,
            another_ignored: String,
        }
    };
    let result = tree::derive_tree_impl(&input);
    assert!(
        result.is_ok(),
        "unannotated fields should be silently ignored"
    );
    let tokens = result.unwrap().to_string();
    // Ignored fields should not appear in the output.
    assert!(
        !tokens.contains("ignored_field"),
        "ignored_field should not appear"
    );
    assert!(
        !tokens.contains("another_ignored"),
        "another_ignored should not appear"
    );
}

// -- tree::TreeAttr parsing --------------------------------------------------

#[test]
fn test_parse_tree_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { style = "bold" };
    let attr: tree::TreeAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "style");
    match attr.value {
        tree::TreeAttrValue::Str(s) => assert_eq!(s.value(), "bold"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_tree_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { style };
    let attr: tree::TreeAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "style");
    matches!(attr.value, tree::TreeAttrValue::Flag);
}

#[test]
fn test_tree_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { guide_style = "dim cyan" };
    let attr: tree::TreeAttr = syn::parse2(tokens).unwrap();
    let result = tree::tree_expect_str(&attr, "guide_style");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "dim cyan");
}

#[test]
fn test_tree_expect_str_wrong_type() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { style };
    let attr: tree::TreeAttr = syn::parse2(tokens).unwrap();
    let result = tree::tree_expect_str(&attr, "style");
    assert!(result.is_err());
}

// -- Renderable derive tests -------------------------------------------

#[test]
fn test_derive_renderable_via_panel() {
    let input: DeriveInput = syn::parse_quote! {
        #[renderable(via = "panel")]
        struct Config {
            host: String,
            port: u16,
        }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(
        result.is_ok(),
        "derive_renderable_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("Renderable"),
        "should implement Renderable trait"
    );
    assert!(
        tokens.contains("gilt_console"),
        "should generate gilt_console method"
    );
    assert!(tokens.contains("to_panel"), "should delegate to to_panel()");
    assert!(
        !tokens.contains("to_tree"),
        "should not reference to_tree()"
    );
}

#[test]
fn test_derive_renderable_via_tree() {
    let input: DeriveInput = syn::parse_quote! {
        #[renderable(via = "tree")]
        struct FileEntry {
            name: String,
            entries: Vec<String>,
        }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(
        result.is_ok(),
        "derive_renderable_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("Renderable"),
        "should implement Renderable trait"
    );
    assert!(
        tokens.contains("gilt_console"),
        "should generate gilt_console method"
    );
    assert!(tokens.contains("to_tree"), "should delegate to to_tree()");
    assert!(
        !tokens.contains("to_panel"),
        "should not reference to_panel()"
    );
}

#[test]
fn test_derive_renderable_default() {
    // No #[renderable(...)] attribute at all — defaults to panel.
    let input: DeriveInput = syn::parse_quote! {
        struct Simple {
            name: String,
            value: u32,
        }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(
        result.is_ok(),
        "derive_renderable_impl should succeed with no attrs"
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("Renderable"),
        "should implement Renderable trait"
    );
    assert!(
        tokens.contains("to_panel"),
        "default delegation should be to_panel()"
    );
}

#[test]
fn test_derive_renderable_rejects_unknown_via() {
    let input: DeriveInput = syn::parse_quote! {
        #[renderable(via = "table")]
        struct Rec {
            a: String,
        }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(result.is_err(), "should reject unknown via value");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("unknown renderable via"),
        "error should mention unknown via, got: {}",
        err_msg
    );
}

#[test]
fn test_derive_renderable_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

#[test]
fn test_derive_renderable_rejects_unknown_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[renderable(nonexistent = "value")]
        struct Rec {
            a: String,
        }
    };
    let result = renderable::derive_renderable_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown renderable attribute"),);
}

// -- renderable::RenderableAttr parsing --------------------------------------------

#[test]
fn test_parse_renderable_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { via = "panel" };
    let attr: renderable::RenderableAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "via");
    match attr.value {
        renderable::RenderableAttrValue::Str(s) => assert_eq!(s.value(), "panel"),
    }
}

#[test]
fn test_renderable_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { via = "tree" };
    let attr: renderable::RenderableAttr = syn::parse2(tokens).unwrap();
    let result = renderable::renderable_expect_str(&attr, "via");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "tree");
}

// -- Columns derive tests ----------------------------------------------

#[test]
fn test_derive_columns_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct ProjectCard {
            name: String,
            status: String,
        }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(
        result.is_ok(),
        "derive_columns_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_card"), "should generate to_card method");
    assert!(
        tokens.contains("to_columns"),
        "should generate to_columns method"
    );
    assert!(
        tokens.contains("\"Name\""),
        "should contain default label 'Name'"
    );
    assert!(
        tokens.contains("\"Status\""),
        "should contain default label 'Status'"
    );
    assert!(tokens.contains("Panel"), "should reference Panel type");
    assert!(tokens.contains("Columns"), "should reference Columns type");
    // Default title should be the struct name.
    assert!(
        tokens.contains("\"ProjectCard\""),
        "default card title should be struct name"
    );
}

#[test]
fn test_derive_columns_with_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        #[columns(
            column_count = 3,
            equal = true,
            expand = true,
            padding = 2,
            title = "My Projects"
        )]
        struct ProjectCard {
            #[field(label = "Project", style = "bold cyan")]
            name: String,
            #[field(label = "Status")]
            status: String,
        }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(
        result.is_ok(),
        "derive_columns_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_card"), "should generate to_card method");
    assert!(
        tokens.contains("to_columns"),
        "should generate to_columns method"
    );
    assert!(
        tokens.contains("\"Project\""),
        "should use custom label 'Project'"
    );
    assert!(
        tokens.contains("bold cyan"),
        "should contain field style markup"
    );
    assert!(
        tokens.contains("\"Status\""),
        "should use custom label 'Status'"
    );
    assert!(tokens.contains("equal"), "should set equal");
    assert!(tokens.contains("expand"), "should set expand");
    assert!(
        tokens.contains("width"),
        "should set width from column_count"
    );
    assert!(tokens.contains("\"My Projects\""), "should contain title");
}

#[test]
fn test_derive_columns_skip_field() {
    let input: DeriveInput = syn::parse_quote! {
        struct Card {
            visible: String,
            #[field(skip)]
            hidden: u64,
            also_visible: i32,
        }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"Visible\""),
        "should include visible field"
    );
    assert!(
        tokens.contains("\"Also Visible\""),
        "should include also_visible field"
    );
    // The hidden field should not appear.
    assert!(!tokens.contains("\"Hidden\""), "should skip hidden field");
    assert!(
        !tokens.contains("hidden"),
        "hidden field ident should not appear"
    );
}

#[test]
fn test_derive_columns_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

// -- columns::ColumnsAttr parsing -----------------------------------------------

#[test]
fn test_parse_columns_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Cols" };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "title");
    match attr.value {
        columns::ColumnsAttrValue::Str(s) => assert_eq!(s.value(), "My Cols"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_columns_attr_bool() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { equal = true };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "equal");
    match attr.value {
        columns::ColumnsAttrValue::Bool(b) => assert!(b.value),
        _ => panic!("expected Bool"),
    }
}

#[test]
fn test_parse_columns_attr_int() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { column_count = 4 };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "column_count");
    match attr.value {
        columns::ColumnsAttrValue::Int(i) => assert_eq!(i.base10_parse::<usize>().unwrap(), 4),
        _ => panic!("expected Int"),
    }
}

#[test]
fn test_parse_columns_attr_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "expand");
    matches!(attr.value, columns::ColumnsAttrValue::Flag);
}

#[test]
fn test_columns_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    let result = columns::columns_expect_str(&attr, "title");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "hello");
}

#[test]
fn test_columns_expect_bool_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    let result = columns::columns_expect_bool(&attr, "expand");
    assert!(result.is_ok());
    assert!(result.unwrap().value);
}

#[test]
fn test_columns_expect_int_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { padding = 3 };
    let attr: columns::ColumnsAttr = syn::parse2(tokens).unwrap();
    let result = columns::columns_expect_int(&attr, "padding");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().base10_parse::<usize>().unwrap(), 3);
}

#[test]
fn test_derive_columns_rejects_unknown_columns_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[columns(nonexistent = "value")]
        struct Rec {
            a: String,
        }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown columns attribute"),);
}

#[test]
fn test_derive_columns_rejects_unknown_field_attr() {
    let input: DeriveInput = syn::parse_quote! {
        struct Rec {
            #[field(nonexistent = "value")]
            a: String,
        }
    };
    let result = columns::derive_columns_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown field attribute"),);
}

// -- Rule derive -------------------------------------------------------

#[test]
fn test_derive_rule_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct Section {
            heading: String,
        }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(
        result.is_ok(),
        "derive_rule_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_rule"), "should generate to_rule method");
    assert!(tokens.contains("Rule"), "should reference Rule type");
    assert!(
        tokens.contains("with_title"),
        "should use with_title constructor"
    );
    // Default title should be the struct name.
    assert!(
        tokens.contains("\"Section\""),
        "default title should be struct name"
    );
}

#[test]
fn test_derive_rule_with_style() {
    let input: DeriveInput = syn::parse_quote! {
        #[rule(style = "bold blue")]
        struct Divider {
            label: String,
        }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(
        result.is_ok(),
        "derive_rule_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_rule"), "should generate to_rule method");
    assert!(
        tokens.contains("Style :: parse"),
        "should parse style string"
    );
    assert!(
        tokens.contains("\"bold blue\""),
        "should contain style value"
    );
}

#[test]
fn test_derive_rule_with_characters() {
    let input: DeriveInput = syn::parse_quote! {
        #[rule(characters = "=")]
        struct Break {
            text: String,
        }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(
        result.is_ok(),
        "derive_rule_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("characters"),
        "should call characters method"
    );
    assert!(tokens.contains("\"=\""), "should contain custom character");
}

#[test]
fn test_derive_rule_with_align() {
    let input: DeriveInput = syn::parse_quote! {
        #[rule(align = "left")]
        struct Header {
            text: String,
        }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(
        result.is_ok(),
        "derive_rule_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("align"), "should call align method");
    assert!(tokens.contains("Left"), "should contain Left variant");
}

#[test]
fn test_derive_rule_title_field() {
    let input: DeriveInput = syn::parse_quote! {
        struct Section {
            #[rule(title)]
            heading: String,
            extra: u32,
        }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(
        result.is_ok(),
        "derive_rule_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("to_rule"), "should generate to_rule method");
    // Should use the field `heading` as the title source.
    assert!(tokens.contains("heading"), "should reference heading field");
    assert!(
        tokens.contains("to_string"),
        "should call to_string on field"
    );
    // Should NOT fall back to struct name.
    assert!(
        !tokens.contains("\"Section\""),
        "should not use struct name as title"
    );
}

#[test]
fn test_derive_rule_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = rule::derive_rule_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

// -- rule::RuleAttr parsing --------------------------------------------------

#[test]
fn test_parse_rule_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { style = "bold red" };
    let attr: rule::RuleAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "style");
    match attr.value {
        rule::RuleAttrValue::Str(s) => assert_eq!(s.value(), "bold red"),
    }
}

#[test]
fn test_rule_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { characters = "─" };
    let attr: rule::RuleAttr = syn::parse2(tokens).unwrap();
    let result = rule::rule_expect_str(&attr, "characters");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "─");
}

// -- Inspect derive tests ----------------------------------------------

#[test]
fn test_derive_inspect_basic() {
    let input: DeriveInput = syn::parse_quote! {
        struct Config {
            host: String,
            port: u16,
        }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(
        result.is_ok(),
        "derive_inspect_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("to_inspect"),
        "should generate to_inspect method"
    );
    assert!(tokens.contains("Inspect"), "should reference Inspect type");
    assert!(tokens.contains("Debug"), "should have Debug bound");
}

#[test]
fn test_derive_inspect_with_title() {
    let input: DeriveInput = syn::parse_quote! {
        #[inspect(title = "Server Info")]
        struct Config {
            host: String,
            port: u16,
        }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(
        result.is_ok(),
        "derive_inspect_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(
        tokens.contains("\"Server Info\""),
        "should contain custom title"
    );
    assert!(tokens.contains("with_title"), "should call with_title");
}

#[test]
fn test_derive_inspect_with_all_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        #[inspect(title = "My Widget", label = "web-01", doc = "A web server", pretty = false)]
        struct Server {
            host: String,
            cpu: f32,
        }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(
        result.is_ok(),
        "derive_inspect_impl failed: {:?}",
        result.err()
    );
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("with_title"), "should call with_title");
    assert!(tokens.contains("with_label"), "should call with_label");
    assert!(tokens.contains("with_doc"), "should call with_doc");
    assert!(tokens.contains("with_pretty"), "should call with_pretty");
}

#[test]
fn test_derive_inspect_pretty_false() {
    let input: DeriveInput = syn::parse_quote! {
        #[inspect(pretty = false)]
        struct Config {
            host: String,
        }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("with_pretty"), "should call with_pretty");
}

#[test]
fn test_derive_inspect_rejects_enum() {
    let input: DeriveInput = syn::parse_quote! {
        enum Foo { A, B }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not support enums"));
}

#[test]
fn test_derive_inspect_rejects_unknown_attr() {
    let input: DeriveInput = syn::parse_quote! {
        #[inspect(nonexistent = "value")]
        struct Rec {
            a: String,
        }
    };
    let result = inspect::derive_inspect_impl(&input);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown inspect attribute"),);
}

// -- inspect::InspectAttr parsing -----------------------------------------------

#[test]
fn test_parse_inspect_attr_str() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Inspect" };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "title");
    match attr.value {
        inspect::InspectAttrValue::Str(s) => assert_eq!(s.value(), "My Inspect"),
        _ => panic!("expected Str"),
    }
}

#[test]
fn test_parse_inspect_attr_bool() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty = false };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    assert_eq!(attr.key, "pretty");
    match attr.value {
        inspect::InspectAttrValue::Bool(b) => assert!(!b.value),
        _ => panic!("expected Bool"),
    }
}

#[test]
fn test_inspect_expect_str_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    let result = inspect::inspect_expect_str(&attr, "title");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), "hello");
}

#[test]
fn test_inspect_expect_str_wrong_type() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    let result = inspect::inspect_expect_str(&attr, "title");
    assert!(result.is_err());
}

#[test]
fn test_inspect_expect_bool_flag() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    let result = inspect::inspect_expect_bool(&attr, "pretty");
    assert!(result.is_ok());
    assert!(result.unwrap().value);
}

#[test]
fn test_inspect_expect_bool_ok() {
    let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty = false };
    let attr: inspect::InspectAttr = syn::parse2(tokens).unwrap();
    let result = inspect::inspect_expect_bool(&attr, "pretty");
    assert!(result.is_ok());
    assert!(!result.unwrap().value);
}

// ===================================================================
// insta snapshot tests — exact-token guards on each derive's output
// ===================================================================
//
// These guard against accidental codegen drift during refactors.
// To regenerate after an intentional change:
//
//     INSTA_UPDATE=always cargo test -p gilt-derive --lib expand_
//
// Snapshots live at crates/gilt-derive/src/snapshots/.

/// Pretty-print a TokenStream so snapshots are readable. We use
/// `prettyplease` only here; if it's unavailable we fall back to the
/// raw string form (still stable since `quote!` produces deterministic
/// output for given inputs).
fn render_tokens(ts: &proc_macro2::TokenStream) -> String {
    // Try prettyplease via syn::parse2 → prettyplease::unparse. If the
    // tokens don't form a complete File (some derives emit bare items),
    // fall back to the raw string.
    match syn::parse2::<syn::File>(ts.clone()) {
        Ok(file) => prettyplease_or_raw(&file, ts),
        Err(_) => ts.to_string(),
    }
}

fn prettyplease_or_raw(file: &syn::File, ts: &proc_macro2::TokenStream) -> String {
    // prettyplease isn't a hard dep — keep the test resilient if it's
    // ever removed. quote!-based output is already deterministic so the
    // raw form is acceptable.
    let _ = file;
    ts.to_string()
}

#[test]
fn expand_table_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Row {
            id: u32,
            name: String,
        }
    };
    let ts = table::derive_table_impl(&input).expect("Table derive must succeed on minimal input");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_panel_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Server {
            host: String,
            port: u16,
        }
    };
    let ts = panel::derive_panel_impl(&input).expect("Panel derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_tree_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Node {
            #[tree(label)]
            label: String,
            #[tree(children)]
            children: Vec<Node>,
        }
    };
    let ts = tree::derive_tree_impl(&input).expect("Tree derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_columns_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Item {
            title: String,
        }
    };
    let ts = columns::derive_columns_impl(&input).expect("Columns derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_rule_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Section {
            #[rule(title)]
            heading: String,
        }
    };
    let ts = rule::derive_rule_impl(&input).expect("Rule derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_inspect_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Status {
            cpu: f64,
            memory: f64,
        }
    };
    let ts = inspect::derive_inspect_impl(&input).expect("Inspect derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}

#[test]
fn expand_renderable_minimal() {
    let input: DeriveInput = syn::parse_quote! {
        struct Card {
            title: String,
        }
    };
    let ts = renderable::derive_renderable_impl(&input).expect("Renderable derive must succeed");
    insta::assert_snapshot!(render_tokens(&ts));
}
