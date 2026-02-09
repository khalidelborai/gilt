//! Derive macros for the gilt terminal formatting library.
//!
//! This crate provides the `#[derive(Table)]`, `#[derive(Panel)]`, `#[derive(Tree)]`,
//! `#[derive(Columns)]`, `#[derive(Rule)]`, `#[derive(Inspect)]`, and `#[derive(Renderable)]` macros that generate widget
//! conversion methods and trait implementations for structs.
//!
//! # Table Example
//!
//! ```ignore
//! use gilt::Table;
//!
//! #[derive(Table)]
//! #[table(title = "Employees", box_style = "ROUNDED", header_style = "bold cyan")]
//! struct Employee {
//!     #[column(header = "Full Name", style = "bold")]
//!     name: String,
//!     #[column(justify = "right")]
//!     age: u32,
//!     #[column(skip)]
//!     internal_id: u64,
//!     #[column(header = "Dept", style = "green", min_width = 10)]
//!     department: String,
//! }
//!
//! let employees = vec![
//!     Employee {
//!         name: "Alice".into(),
//!         age: 30,
//!         internal_id: 1001,
//!         department: "Engineering".into(),
//!     },
//!     Employee {
//!         name: "Bob".into(),
//!         age: 25,
//!         internal_id: 1002,
//!         department: "Marketing".into(),
//!     },
//! ];
//! let table = Employee::to_table(&employees);
//! ```
//!
//! # Panel Example
//!
//! ```ignore
//! use gilt::Panel;
//!
//! #[derive(Panel)]
//! #[panel(title = "Server Status", box_style = "ROUNDED", border_style = "blue")]
//! struct ServerStatus {
//!     #[field(label = "Host", style = "bold cyan")]
//!     name: String,
//!     #[field(label = "CPU %", style = "yellow")]
//!     cpu: f32,
//!     #[field(skip)]
//!     internal_id: u64,
//!     #[field(label = "Memory", style = "green")]
//!     memory: f32,
//! }
//!
//! let status = ServerStatus {
//!     name: "web-01".into(),
//!     cpu: 42.5,
//!     internal_id: 1001,
//!     memory: 67.3,
//! };
//! let panel = status.to_panel();
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, LitBool, LitInt, LitStr, Token};

// ---------------------------------------------------------------------------
// snake_to_title_case
// ---------------------------------------------------------------------------

/// Convert a snake_case identifier to Title Case.
///
/// Examples:
/// - `first_name` -> "First Name"
/// - `age` -> "Age"
/// - `department_id` -> "Department Id"
fn snake_to_title_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Struct-level attribute: #[table(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[table(...)]` attributes.
#[derive(Default)]
struct TableAttrs {
    title: Option<LitStr>,
    caption: Option<LitStr>,
    box_style: Option<LitStr>,
    style: Option<LitStr>,
    border_style: Option<LitStr>,
    header_style: Option<LitStr>,
    title_style: Option<LitStr>,
    caption_style: Option<LitStr>,
    show_header: Option<LitBool>,
    show_lines: Option<LitBool>,
    show_edge: Option<LitBool>,
    pad_edge: Option<LitBool>,
    expand: Option<LitBool>,
    highlight: Option<LitBool>,
    row_styles: Option<LitStr>,
}

/// A single key=value (or standalone bool key) inside `#[table(...)]`.
struct TableAttr {
    key: Ident,
    value: TableAttrValue,
}

enum TableAttrValue {
    Str(LitStr),
    Bool(LitBool),
    /// Standalone flag like `expand` (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for TableAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(TableAttr {
                    key,
                    value: TableAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(TableAttr {
                    key,
                    value: TableAttrValue::Bool(lit),
                })
            } else {
                Err(input.error("expected string literal or bool"))
            }
        } else {
            // Standalone flag
            Ok(TableAttr {
                key,
                value: TableAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[table(...)]` attributes from a `DeriveInput`.
fn parse_table_attrs(input: &DeriveInput) -> syn::Result<TableAttrs> {
    let mut attrs = TableAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("table") {
            continue;
        }
        let items: Punctuated<TableAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "title" => {
                    attrs.title = Some(expect_str(&item, "title")?);
                }
                "caption" => {
                    attrs.caption = Some(expect_str(&item, "caption")?);
                }
                "box_style" => {
                    attrs.box_style = Some(expect_str(&item, "box_style")?);
                }
                "style" => {
                    attrs.style = Some(expect_str(&item, "style")?);
                }
                "border_style" => {
                    attrs.border_style = Some(expect_str(&item, "border_style")?);
                }
                "header_style" => {
                    attrs.header_style = Some(expect_str(&item, "header_style")?);
                }
                "title_style" => {
                    attrs.title_style = Some(expect_str(&item, "title_style")?);
                }
                "caption_style" => {
                    attrs.caption_style = Some(expect_str(&item, "caption_style")?);
                }
                "show_header" => {
                    attrs.show_header = Some(expect_bool(&item, "show_header")?);
                }
                "show_lines" => {
                    attrs.show_lines = Some(expect_bool(&item, "show_lines")?);
                }
                "show_edge" => {
                    attrs.show_edge = Some(expect_bool(&item, "show_edge")?);
                }
                "pad_edge" => {
                    attrs.pad_edge = Some(expect_bool(&item, "pad_edge")?);
                }
                "expand" => {
                    attrs.expand = Some(expect_bool(&item, "expand")?);
                }
                "highlight" => {
                    attrs.highlight = Some(expect_bool(&item, "highlight")?);
                }
                "row_styles" => {
                    attrs.row_styles = Some(expect_str(&item, "row_styles")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown table attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn expect_str(attr: &TableAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        TableAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn expect_bool(attr: &TableAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        TableAttrValue::Bool(b) => Ok(b.clone()),
        TableAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Field-level attribute: #[column(...)]
// ---------------------------------------------------------------------------

/// Parsed field-level `#[column(...)]` attributes.
#[derive(Default)]
struct ColumnAttrs {
    header: Option<LitStr>,
    style: Option<LitStr>,
    header_style: Option<LitStr>,
    justify: Option<LitStr>,
    width: Option<LitInt>,
    min_width: Option<LitInt>,
    max_width: Option<LitInt>,
    no_wrap: Option<LitBool>,
    skip: Option<LitBool>,
    ratio: Option<LitInt>,
}

/// A single key=value (or standalone flag) inside `#[column(...)]`.
struct ColumnAttr {
    key: Ident,
    value: ColumnAttrValue,
}

enum ColumnAttrValue {
    Str(LitStr),
    Bool(LitBool),
    Int(LitInt),
    /// Standalone flag like `skip` (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for ColumnAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(ColumnAttr {
                    key,
                    value: ColumnAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(ColumnAttr {
                    key,
                    value: ColumnAttrValue::Bool(lit),
                })
            } else if input.peek(LitInt) {
                let lit: LitInt = input.parse()?;
                Ok(ColumnAttr {
                    key,
                    value: ColumnAttrValue::Int(lit),
                })
            } else {
                Err(input.error("expected string literal, bool, or integer"))
            }
        } else {
            // Standalone flag (e.g. `skip`, `no_wrap`)
            Ok(ColumnAttr {
                key,
                value: ColumnAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[column(...)]` attributes from a field.
fn parse_column_attrs(field: &syn::Field) -> syn::Result<ColumnAttrs> {
    let mut attrs = ColumnAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("column") {
            continue;
        }
        let items: Punctuated<ColumnAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "header" => {
                    attrs.header = Some(col_expect_str(&item, "header")?);
                }
                "style" => {
                    attrs.style = Some(col_expect_str(&item, "style")?);
                }
                "header_style" => {
                    attrs.header_style = Some(col_expect_str(&item, "header_style")?);
                }
                "justify" => {
                    attrs.justify = Some(col_expect_str(&item, "justify")?);
                }
                "width" => {
                    attrs.width = Some(col_expect_int(&item, "width")?);
                }
                "min_width" => {
                    attrs.min_width = Some(col_expect_int(&item, "min_width")?);
                }
                "max_width" => {
                    attrs.max_width = Some(col_expect_int(&item, "max_width")?);
                }
                "no_wrap" => {
                    attrs.no_wrap = Some(col_expect_bool(&item, "no_wrap")?);
                }
                "skip" => {
                    attrs.skip = Some(col_expect_bool(&item, "skip")?);
                }
                "ratio" => {
                    attrs.ratio = Some(col_expect_int(&item, "ratio")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown column attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn col_expect_str(attr: &ColumnAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        ColumnAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn col_expect_bool(attr: &ColumnAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        ColumnAttrValue::Bool(b) => Ok(b.clone()),
        ColumnAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

fn col_expect_int(attr: &ColumnAttr, name: &str) -> syn::Result<LitInt> {
    match &attr.value {
        ColumnAttrValue::Int(i) => Ok(i.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects an integer literal", name),
        )),
    }
}

// ---------------------------------------------------------------------------
// box_style -> token mapping
// ---------------------------------------------------------------------------

/// Map a `box_style` string literal to a token stream referencing the
/// corresponding `gilt::box_chars::*` static.
fn box_style_tokens(lit: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    let val = lit.value();
    let ident_str = match val.as_str() {
        "ASCII" => "ASCII",
        "ASCII2" => "ASCII2",
        "ASCII_DOUBLE_HEAD" => "ASCII_DOUBLE_HEAD",
        "SQUARE" => "SQUARE",
        "SQUARE_DOUBLE_HEAD" => "SQUARE_DOUBLE_HEAD",
        "MINIMAL" => "MINIMAL",
        "MINIMAL_HEAVY_HEAD" => "MINIMAL_HEAVY_HEAD",
        "MINIMAL_DOUBLE_HEAD" => "MINIMAL_DOUBLE_HEAD",
        "SIMPLE" => "SIMPLE",
        "SIMPLE_HEAD" => "SIMPLE_HEAD",
        "SIMPLE_HEAVY" => "SIMPLE_HEAVY",
        "HORIZONTALS" => "HORIZONTALS",
        "ROUNDED" => "ROUNDED",
        "HEAVY" => "HEAVY",
        "HEAVY_EDGE" => "HEAVY_EDGE",
        "HEAVY_HEAD" => "HEAVY_HEAD",
        "DOUBLE" => "DOUBLE",
        "DOUBLE_EDGE" => "DOUBLE_EDGE",
        "MARKDOWN" => "MARKDOWN",
        other => {
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "unknown box_style `{other}`. Expected one of: ASCII, ASCII2, \
                     ASCII_DOUBLE_HEAD, SQUARE, SQUARE_DOUBLE_HEAD, MINIMAL, \
                     MINIMAL_HEAVY_HEAD, MINIMAL_DOUBLE_HEAD, SIMPLE, SIMPLE_HEAD, \
                     SIMPLE_HEAVY, HORIZONTALS, ROUNDED, HEAVY, HEAVY_EDGE, HEAVY_HEAD, \
                     DOUBLE, DOUBLE_EDGE, MARKDOWN"
                ),
            ));
        }
    };
    let ident = Ident::new(ident_str, Span::call_site());
    Ok(quote! { Some(&*gilt::box_chars::#ident) })
}

// ---------------------------------------------------------------------------
// justify -> token mapping
// ---------------------------------------------------------------------------

/// Map a `justify` string literal to a token stream for `gilt::text::JustifyMethod`.
fn justify_tokens(lit: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    let val = lit.value();
    match val.as_str() {
        "left" => Ok(quote! { gilt::text::JustifyMethod::Left }),
        "center" => Ok(quote! { gilt::text::JustifyMethod::Center }),
        "right" => Ok(quote! { gilt::text::JustifyMethod::Right }),
        "full" => Ok(quote! { gilt::text::JustifyMethod::Full }),
        other => Err(syn::Error::new_spanned(
            lit,
            format!("unknown justify `{other}`. Expected one of: left, center, right, full"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Derive macro
// ---------------------------------------------------------------------------

/// Derive macro that generates a `to_table(items: &[Self]) -> gilt::table::Table` method.
///
/// # Struct-level attributes (`#[table(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `title` | string | Custom table title (default: struct name) |
/// | `caption` | string | Table caption |
/// | `box_style` | string | Box chars preset (e.g. "ROUNDED", "HEAVY") |
/// | `style` | string | Table-level style string |
/// | `border_style` | string | Border style |
/// | `header_style` | string | Header row style |
/// | `title_style` | string | Title style |
/// | `caption_style` | string | Caption style |
/// | `show_header` | bool | Show/hide header (default true) |
/// | `show_lines` | bool | Show row separators |
/// | `show_edge` | bool | Show outer border |
/// | `pad_edge` | bool | Pad outer edges |
/// | `expand` | bool | Expand to fill width |
/// | `highlight` | bool | Enable highlighting |
/// | `row_styles` | string | Comma-separated alternating row styles |
///
/// # Field-level attributes (`#[column(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `header` | string | Custom column header (default: Title Case field name) |
/// | `style` | string | Column style |
/// | `header_style` | string | Column header style |
/// | `justify` | string | "left", "center", "right", "full" |
/// | `width` | int | Fixed column width |
/// | `min_width` | int | Minimum column width |
/// | `max_width` | int | Maximum column width |
/// | `no_wrap` | bool | Disable wrapping |
/// | `skip` | bool | Exclude field from table |
/// | `ratio` | int | Column width ratio |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Table;
///
/// #[derive(Table)]
/// #[table(title = "Employees", box_style = "ROUNDED", header_style = "bold cyan")]
/// struct Employee {
///     #[column(header = "Full Name", style = "bold")]
///     name: String,
///     #[column(justify = "right")]
///     age: u32,
///     #[column(skip)]
///     internal_id: u64,
///     #[column(header = "Dept", style = "green", min_width = 10)]
///     department: String,
/// }
///
/// let employees = vec![
///     Employee {
///         name: "Alice".into(),
///         age: 30,
///         internal_id: 1001,
///         department: "Engineering".into(),
///     },
/// ];
/// let table = Employee::to_table(&employees);
/// ```
#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_table_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_table_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Only support structs with named fields.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Table derive only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Table derive does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Table derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Table derive does not support unions",
            ));
        }
    };

    // Parse struct-level #[table(...)] attributes.
    let table_attrs = parse_table_attrs(input)?;

    // Collect field info, respecting `skip`.
    struct FieldInfo {
        ident: Ident,
        header: String,
        col_attrs: ColumnAttrs,
    }
    let mut field_infos: Vec<FieldInfo> = Vec::new();

    for field in fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .clone();
        let col_attrs = parse_column_attrs(field)?;

        // Check skip.
        let skip = col_attrs.skip.as_ref().map(|b| b.value).unwrap_or(false);
        if skip {
            continue;
        }

        let header = match &col_attrs.header {
            Some(lit) => lit.value(),
            None => snake_to_title_case(&ident.to_string()),
        };

        field_infos.push(FieldInfo {
            ident,
            header,
            col_attrs,
        });
    }

    // Build header string literals.
    let header_strs: Vec<&str> = field_infos.iter().map(|fi| fi.header.as_str()).collect();
    let header_tokens = header_strs.iter().map(|h| quote! { #h });

    // Build the title token -- use custom title or fall back to struct name.
    let title_value = match &table_attrs.title {
        Some(lit) => lit.value(),
        None => struct_name_str.clone(),
    };

    // Build table-level configuration statements.
    let mut table_config = Vec::new();

    // Title is always set.
    table_config.push(quote! {
        table.title = Some(#title_value.to_string());
    });

    if let Some(ref lit) = table_attrs.caption {
        let val = lit.value();
        table_config.push(quote! {
            table.caption = Some(#val.to_string());
        });
    }
    if let Some(ref lit) = table_attrs.box_style {
        let tokens = box_style_tokens(lit)?;
        table_config.push(quote! {
            table.box_chars = #tokens;
        });
    }
    if let Some(ref lit) = table_attrs.style {
        let val = lit.value();
        table_config.push(quote! {
            table.style = #val.to_string();
        });
    }
    if let Some(ref lit) = table_attrs.border_style {
        let val = lit.value();
        table_config.push(quote! {
            table.border_style = #val.to_string();
        });
    }
    if let Some(ref lit) = table_attrs.header_style {
        let val = lit.value();
        table_config.push(quote! {
            table.header_style = #val.to_string();
        });
    }
    if let Some(ref lit) = table_attrs.title_style {
        let val = lit.value();
        table_config.push(quote! {
            table.title_style = #val.to_string();
        });
    }
    if let Some(ref lit) = table_attrs.caption_style {
        let val = lit.value();
        table_config.push(quote! {
            table.caption_style = #val.to_string();
        });
    }
    if let Some(ref lit) = table_attrs.show_header {
        let val = lit.value;
        table_config.push(quote! {
            table.show_header = #val;
        });
    }
    if let Some(ref lit) = table_attrs.show_lines {
        let val = lit.value;
        table_config.push(quote! {
            table.show_lines = #val;
        });
    }
    if let Some(ref lit) = table_attrs.show_edge {
        let val = lit.value;
        table_config.push(quote! {
            table.show_edge = #val;
        });
    }
    if let Some(ref lit) = table_attrs.pad_edge {
        let val = lit.value;
        table_config.push(quote! {
            table.pad_edge = #val;
        });
    }
    if let Some(ref lit) = table_attrs.expand {
        let val = lit.value;
        table_config.push(quote! {
            table.set_expand(#val);
        });
    }
    if let Some(ref lit) = table_attrs.highlight {
        let val = lit.value;
        table_config.push(quote! {
            table.highlight = #val;
        });
    }
    if let Some(ref lit) = table_attrs.row_styles {
        let val = lit.value();
        let styles: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
        table_config.push(quote! {
            table.row_styles = vec![#(#styles.to_string()),*];
        });
    }

    // Build per-column configuration statements.
    let mut col_configs = Vec::new();
    for (i, fi) in field_infos.iter().enumerate() {
        let ca = &fi.col_attrs;

        if let Some(ref lit) = ca.style {
            let val = lit.value();
            col_configs.push(quote! {
                table.columns[#i].style = #val.to_string();
            });
        }
        if let Some(ref lit) = ca.header_style {
            let val = lit.value();
            col_configs.push(quote! {
                table.columns[#i].header_style = #val.to_string();
            });
        }
        if let Some(ref lit) = ca.justify {
            let tokens = justify_tokens(lit)?;
            col_configs.push(quote! {
                table.columns[#i].justify = #tokens;
            });
        }
        if let Some(ref lit) = ca.width {
            let val: usize = lit.base10_parse()?;
            col_configs.push(quote! {
                table.columns[#i].width = Some(#val);
            });
        }
        if let Some(ref lit) = ca.min_width {
            let val: usize = lit.base10_parse()?;
            col_configs.push(quote! {
                table.columns[#i].min_width = Some(#val);
            });
        }
        if let Some(ref lit) = ca.max_width {
            let val: usize = lit.base10_parse()?;
            col_configs.push(quote! {
                table.columns[#i].max_width = Some(#val);
            });
        }
        if let Some(ref lit) = ca.no_wrap {
            let val = lit.value;
            col_configs.push(quote! {
                table.columns[#i].no_wrap = #val;
            });
        }
        if let Some(ref lit) = ca.ratio {
            let val: usize = lit.base10_parse()?;
            col_configs.push(quote! {
                table.columns[#i].ratio = Some(#val);
            });
        }
    }

    // Build row expression: for each non-skipped field, push `&item.field.to_string()`.
    let row_fields = field_infos.iter().map(|fi| {
        let ident = &fi.ident;
        quote! { &item.#ident.to_string() }
    });

    let expanded = quote! {
        impl #struct_name {
            /// Creates a [`gilt::table::Table`] from a slice of items.
            ///
            /// Each non-skipped struct field becomes a column, with headers derived
            /// from field names converted to Title Case (or overridden via
            /// `#[column(header = "...")]`). Struct-level `#[table(...)]` attributes
            /// control the overall table appearance.
            pub fn to_table(items: &[Self]) -> gilt::table::Table {
                let mut table = gilt::table::Table::new(&[#(#header_tokens),*]);
                #(#table_config)*
                #(#col_configs)*
                for item in items {
                    table.add_row(&[#(#row_fields),*]);
                }
                table
            }
        }
    };

    Ok(expanded)
}

// ===========================================================================
// Panel derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[panel(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[panel(...)]` attributes.
#[derive(Default)]
struct PanelAttrs {
    title: Option<LitStr>,
    subtitle: Option<LitStr>,
    box_style: Option<LitStr>,
    border_style: Option<LitStr>,
    style: Option<LitStr>,
    title_style: Option<LitStr>,
    expand: Option<LitBool>,
    highlight: Option<LitBool>,
}

/// A single key=value (or standalone bool key) inside `#[panel(...)]`.
struct PanelAttr {
    key: Ident,
    value: PanelAttrValue,
}

enum PanelAttrValue {
    Str(LitStr),
    Bool(LitBool),
    /// Standalone flag like `expand` (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for PanelAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(PanelAttr {
                    key,
                    value: PanelAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(PanelAttr {
                    key,
                    value: PanelAttrValue::Bool(lit),
                })
            } else {
                Err(input.error("expected string literal or bool"))
            }
        } else {
            // Standalone flag
            Ok(PanelAttr {
                key,
                value: PanelAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[panel(...)]` attributes from a `DeriveInput`.
fn parse_panel_attrs(input: &DeriveInput) -> syn::Result<PanelAttrs> {
    let mut attrs = PanelAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("panel") {
            continue;
        }
        let items: Punctuated<PanelAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "title" => {
                    attrs.title = Some(panel_expect_str(&item, "title")?);
                }
                "subtitle" => {
                    attrs.subtitle = Some(panel_expect_str(&item, "subtitle")?);
                }
                "box_style" => {
                    attrs.box_style = Some(panel_expect_str(&item, "box_style")?);
                }
                "border_style" => {
                    attrs.border_style = Some(panel_expect_str(&item, "border_style")?);
                }
                "style" => {
                    attrs.style = Some(panel_expect_str(&item, "style")?);
                }
                "title_style" => {
                    attrs.title_style = Some(panel_expect_str(&item, "title_style")?);
                }
                "expand" => {
                    attrs.expand = Some(panel_expect_bool(&item, "expand")?);
                }
                "highlight" => {
                    attrs.highlight = Some(panel_expect_bool(&item, "highlight")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown panel attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn panel_expect_str(attr: &PanelAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        PanelAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn panel_expect_bool(attr: &PanelAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        PanelAttrValue::Bool(b) => Ok(b.clone()),
        PanelAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Field-level attribute: #[field(...)]
// ---------------------------------------------------------------------------

/// Parsed field-level `#[field(...)]` attributes.
#[derive(Default)]
struct FieldAttrs {
    label: Option<LitStr>,
    style: Option<LitStr>,
    skip: Option<LitBool>,
}

/// A single key=value (or standalone flag) inside `#[field(...)]`.
struct FieldAttr {
    key: Ident,
    value: FieldAttrValue,
}

enum FieldAttrValue {
    Str(LitStr),
    Bool(LitBool),
    /// Standalone flag like `skip` (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(FieldAttr {
                    key,
                    value: FieldAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(FieldAttr {
                    key,
                    value: FieldAttrValue::Bool(lit),
                })
            } else {
                Err(input.error("expected string literal or bool"))
            }
        } else {
            // Standalone flag
            Ok(FieldAttr {
                key,
                value: FieldAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[field(...)]` attributes from a field.
fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("field") {
            continue;
        }
        let items: Punctuated<FieldAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "label" => {
                    attrs.label = Some(field_expect_str(&item, "label")?);
                }
                "style" => {
                    attrs.style = Some(field_expect_str(&item, "style")?);
                }
                "skip" => {
                    attrs.skip = Some(field_expect_bool(&item, "skip")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown field attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn field_expect_str(attr: &FieldAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        FieldAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn field_expect_bool(attr: &FieldAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        FieldAttrValue::Bool(b) => Ok(b.clone()),
        FieldAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Panel derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates a `to_panel(&self) -> gilt::panel::Panel` method.
///
/// # Struct-level attributes (`#[panel(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `title` | string | Custom panel title (default: struct name) |
/// | `subtitle` | string | Panel subtitle |
/// | `box_style` | string | Box chars preset (e.g. "ROUNDED", "HEAVY") |
/// | `border_style` | string | Border style |
/// | `style` | string | Content area style string |
/// | `title_style` | string | Title style |
/// | `expand` | bool | Expand to fill width (default true) |
/// | `highlight` | bool | Enable highlighting |
///
/// # Field-level attributes (`#[field(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `label` | string | Custom field label (default: Title Case field name) |
/// | `style` | string | Style applied as markup around the label |
/// | `skip` | bool | Exclude field from panel |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Panel;
///
/// #[derive(Panel)]
/// #[panel(title = "Server Status", box_style = "ROUNDED", border_style = "blue")]
/// struct ServerStatus {
///     #[field(label = "Host", style = "bold cyan")]
///     name: String,
///     #[field(label = "CPU %", style = "yellow")]
///     cpu: f32,
///     #[field(skip)]
///     internal_id: u64,
///     #[field(label = "Memory", style = "green")]
///     memory: f32,
/// }
///
/// let status = ServerStatus {
///     name: "web-01".into(),
///     cpu: 42.5,
///     internal_id: 1001,
///     memory: 67.3,
/// };
/// let panel = status.to_panel();
/// ```
#[proc_macro_derive(Panel, attributes(panel, field))]
pub fn derive_panel(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_panel_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_panel_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Only support structs with named fields.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Panel derive only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Panel derive does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Panel derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Panel derive does not support unions",
            ));
        }
    };

    // Parse struct-level #[panel(...)] attributes.
    let panel_attrs = parse_panel_attrs(input)?;

    // Collect field info, respecting `skip`.
    struct PanelFieldInfo {
        ident: Ident,
        label: String,
        style: Option<String>,
    }
    let mut field_infos: Vec<PanelFieldInfo> = Vec::new();

    for field in fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .clone();
        let fa = parse_field_attrs(field)?;

        // Check skip.
        let skip = fa.skip.as_ref().map(|b| b.value).unwrap_or(false);
        if skip {
            continue;
        }

        let label = match &fa.label {
            Some(lit) => lit.value(),
            None => snake_to_title_case(&ident.to_string()),
        };

        let style = fa.style.as_ref().map(|lit| lit.value());

        field_infos.push(PanelFieldInfo {
            ident,
            label,
            style,
        });
    }

    // Build the line push expressions for each field.
    let line_pushes: Vec<proc_macro2::TokenStream> = field_infos
        .iter()
        .map(|fi| {
            let ident = &fi.ident;
            let label = &fi.label;
            match &fi.style {
                Some(sty) => {
                    // "[style]Label:[/style] {value}"
                    let open_tag = format!("[{}]", sty);
                    let close_tag = format!("[/{}]", sty);
                    quote! {
                        lines.push(format!("{}{}:{} {}", #open_tag, #label, #close_tag, self.#ident));
                    }
                }
                None => {
                    // "Label: {value}"
                    quote! {
                        lines.push(format!("{}: {}", #label, self.#ident));
                    }
                }
            }
        })
        .collect();

    // Build the title -- use custom title or fall back to struct name.
    let title_value = match &panel_attrs.title {
        Some(lit) => lit.value(),
        None => struct_name_str.clone(),
    };

    // Build panel configuration statements.
    let mut panel_config = Vec::new();

    // Title is always set (as Text with optional title_style markup).
    if let Some(ref lit) = panel_attrs.title_style {
        let sty = lit.value();
        let styled_title = format!("[{}]{}[/{}]", sty, title_value, sty);
        panel_config.push(quote! {
            panel.title = Some(gilt::text::Text::from_markup(#styled_title).unwrap_or_else(|_| gilt::text::Text::from(#title_value)));
        });
    } else {
        panel_config.push(quote! {
            panel.title = Some(gilt::text::Text::from(#title_value));
        });
    }

    if let Some(ref lit) = panel_attrs.subtitle {
        let val = lit.value();
        panel_config.push(quote! {
            panel.subtitle = Some(gilt::text::Text::from(#val));
        });
    }
    if let Some(ref lit) = panel_attrs.box_style {
        let tokens = box_style_tokens(lit)?;
        // box_style_tokens returns `Some(&*gilt::box_chars::IDENT)`, but panel.box_chars
        // expects `&'static BoxChars` not `Option`. Unwrap the Some.
        panel_config.push(quote! {
            if let Some(bc) = #tokens {
                panel.box_chars = bc;
            }
        });
    }
    if let Some(ref lit) = panel_attrs.border_style {
        let val = lit.value();
        panel_config.push(quote! {
            panel.border_style = gilt::style::Style::parse(#val).unwrap_or_else(|_| gilt::style::Style::null());
        });
    }
    if let Some(ref lit) = panel_attrs.style {
        let val = lit.value();
        panel_config.push(quote! {
            panel.style = gilt::style::Style::parse(#val).unwrap_or_else(|_| gilt::style::Style::null());
        });
    }
    if let Some(ref lit) = panel_attrs.expand {
        let val = lit.value;
        panel_config.push(quote! {
            panel.expand = #val;
        });
    }
    if let Some(ref lit) = panel_attrs.highlight {
        let val = lit.value;
        panel_config.push(quote! {
            panel.highlight = #val;
        });
    }

    let expanded = quote! {
        impl #struct_name {
            /// Creates a [`gilt::panel::Panel`] displaying this struct's fields
            /// as labeled key-value pairs.
            ///
            /// Each non-skipped field becomes a line `"Label: value"`. Field styles
            /// are applied as markup tags around the label. The panel title defaults
            /// to the struct name unless overridden via `#[panel(title = "...")]`.
            pub fn to_panel(&self) -> gilt::panel::Panel {
                let mut lines: Vec<String> = Vec::new();
                #(#line_pushes)*
                let content = gilt::text::Text::from_markup(&lines.join("\n"))
                    .unwrap_or_else(|_| gilt::text::Text::from(lines.join("\n").as_str()));
                let mut panel = gilt::panel::Panel::new(content);
                #(#panel_config)*
                panel
            }
        }
    };

    Ok(expanded)
}

// ===========================================================================
// Tree derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[tree(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[tree(...)]` attributes.
#[derive(Default)]
struct TreeAttrs {
    style: Option<LitStr>,
    guide_style: Option<LitStr>,
}

/// A single key=value inside `#[tree(...)]` at the struct level.
struct TreeAttr {
    key: Ident,
    value: TreeAttrValue,
}

enum TreeAttrValue {
    Str(LitStr),
    /// Standalone flag (no `= ...`).
    Flag,
}

impl Parse for TreeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(TreeAttr {
                    key,
                    value: TreeAttrValue::Str(lit),
                })
            } else {
                Err(input.error("expected string literal"))
            }
        } else {
            Ok(TreeAttr {
                key,
                value: TreeAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[tree(...)]` attributes from a `DeriveInput`.
fn parse_tree_attrs(input: &DeriveInput) -> syn::Result<TreeAttrs> {
    let mut attrs = TreeAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("tree") {
            continue;
        }
        let items: Punctuated<TreeAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "style" => {
                    attrs.style = Some(tree_expect_str(&item, "style")?);
                }
                "guide_style" => {
                    attrs.guide_style = Some(tree_expect_str(&item, "guide_style")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown tree attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn tree_expect_str(attr: &TreeAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        TreeAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Tree derive: field-level attributes #[tree(label)], #[tree(children)], etc.
// ---------------------------------------------------------------------------

/// The role of a field in the Tree derive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeFieldKind {
    Label,
    Children,
    Leaf,
    None,
}

/// Parse `#[tree(...)]` attributes on a field to determine its role.
fn parse_tree_field_attrs(field: &syn::Field) -> syn::Result<TreeFieldKind> {
    let mut kind = TreeFieldKind::None;

    for attr in &field.attrs {
        if !attr.path().is_ident("tree") {
            continue;
        }
        // Parse as a single ident (label, children, leaf) -- no key=value pairs.
        let ident: Ident = attr.parse_args()?;
        let ident_str = ident.to_string();
        match ident_str.as_str() {
            "label" => {
                if kind != TreeFieldKind::None {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        "field already has a tree role assigned",
                    ));
                }
                kind = TreeFieldKind::Label;
            }
            "children" => {
                if kind != TreeFieldKind::None {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        "field already has a tree role assigned",
                    ));
                }
                kind = TreeFieldKind::Children;
            }
            "leaf" => {
                if kind != TreeFieldKind::None {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        "field already has a tree role assigned",
                    ));
                }
                kind = TreeFieldKind::Leaf;
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &ident,
                    format!(
                        "unknown tree field attribute `{}`. Expected one of: label, children, leaf",
                        ident_str
                    ),
                ));
            }
        }
    }

    Ok(kind)
}

// ---------------------------------------------------------------------------
// Tree derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates a `to_tree(&self) -> gilt::tree::Tree` method.
///
/// # Struct-level attributes (`#[tree(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `style` | string | Style string for tree nodes (e.g. "bold") |
/// | `guide_style` | string | Style string for guide lines (e.g. "dim cyan") |
///
/// # Field-level attributes (`#[tree(...)]`)
///
/// | Attribute | Description |
/// |-----------|-------------|
/// | `label` | The field whose `.to_string()` becomes the node label (required, exactly one) |
/// | `children` | The field containing child nodes as `Vec<Self>` (required, exactly one) |
/// | `leaf` | Optional fields shown as leaf text ("FieldName: value") under the node |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Tree;
///
/// #[derive(Tree)]
/// #[tree(style = "bold", guide_style = "dim cyan")]
/// struct FileEntry {
///     #[tree(label)]
///     name: String,
///     #[tree(children)]
///     entries: Vec<FileEntry>,
///     #[tree(leaf)]
///     size: u64,
/// }
///
/// let root = FileEntry {
///     name: "src".into(),
///     entries: vec![
///         FileEntry {
///             name: "main.rs".into(),
///             entries: vec![],
///             size: 1024,
///         },
///     ],
///     size: 0,
/// };
/// let tree = root.to_tree();
/// ```
#[proc_macro_derive(Tree, attributes(tree))]
pub fn derive_tree(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_tree_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_tree_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Only support structs with named fields.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Tree derive only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Tree derive does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Tree derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Tree derive does not support unions",
            ));
        }
    };

    // Parse struct-level #[tree(...)] attributes.
    let tree_attrs = parse_tree_attrs(input)?;

    // Classify fields by their tree role.
    let mut label_field: Option<Ident> = None;
    let mut children_field: Option<Ident> = None;
    let mut leaf_fields: Vec<Ident> = Vec::new();

    for field in fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .clone();
        let kind = parse_tree_field_attrs(field)?;

        match kind {
            TreeFieldKind::Label => {
                if label_field.is_some() {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        "only one field can be marked #[tree(label)]",
                    ));
                }
                label_field = Some(ident);
            }
            TreeFieldKind::Children => {
                if children_field.is_some() {
                    return Err(syn::Error::new_spanned(
                        &ident,
                        "only one field can be marked #[tree(children)]",
                    ));
                }
                children_field = Some(ident);
            }
            TreeFieldKind::Leaf => {
                leaf_fields.push(ident);
            }
            TreeFieldKind::None => {
                // Ignored field.
            }
        }
    }

    // Validate required fields.
    let label_ident = label_field.ok_or_else(|| {
        syn::Error::new_spanned(
            struct_name,
            "Tree derive requires exactly one field marked #[tree(label)]",
        )
    })?;

    let children_ident = children_field.ok_or_else(|| {
        syn::Error::new_spanned(
            struct_name,
            "Tree derive requires exactly one field marked #[tree(children)]",
        )
    })?;

    // Build style configuration.
    let style_setup = if let Some(ref lit) = tree_attrs.style {
        let val = lit.value();
        quote! {
            if let Ok(s) = gilt::style::Style::parse(#val) {
                tree.style = s;
            }
        }
    } else {
        quote! {}
    };

    let guide_style_setup = if let Some(ref lit) = tree_attrs.guide_style {
        let val = lit.value();
        quote! {
            if let Ok(s) = gilt::style::Style::parse(#val) {
                tree.guide_style = s;
            }
        }
    } else {
        quote! {}
    };

    // Build leaf additions.
    let leaf_additions: Vec<proc_macro2::TokenStream> = leaf_fields
        .iter()
        .map(|ident| {
            let leaf_label = snake_to_title_case(&ident.to_string());
            quote! {
                {
                    let leaf_text = format!("{}: {}", #leaf_label, self.#ident);
                    let leaf_label_text = gilt::text::Text::from(leaf_text.as_str());
                    tree.add(leaf_label_text);
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #struct_name {
            /// Creates a [`gilt::tree::Tree`] from this struct.
            ///
            /// The field marked `#[tree(label)]` becomes the node label,
            /// fields marked `#[tree(leaf)]` become leaf nodes, and the
            /// field marked `#[tree(children)]` is recursively converted
            /// to child trees.
            pub fn to_tree(&self) -> gilt::tree::Tree {
                let label_text = gilt::text::Text::from(self.#label_ident.to_string().as_str());
                let mut tree = gilt::tree::Tree::new(label_text);
                #style_setup
                #guide_style_setup
                #(#leaf_additions)*
                for child in &self.#children_ident {
                    let child_tree = child.to_tree();
                    tree.children.push(child_tree);
                }
                tree
            }
        }
    };

    Ok(expanded)
}

// ===========================================================================
// Renderable derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[renderable(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[renderable(...)]` attributes.
#[derive(Default)]
struct RenderableAttrs {
    /// Which widget to delegate to: "panel" or "tree". Defaults to "panel".
    via: Option<LitStr>,
}

/// A single key=value inside `#[renderable(...)]`.
struct RenderableAttr {
    key: Ident,
    value: RenderableAttrValue,
}

enum RenderableAttrValue {
    Str(LitStr),
}

impl Parse for RenderableAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(RenderableAttr {
                    key,
                    value: RenderableAttrValue::Str(lit),
                })
            } else {
                Err(input.error("expected string literal"))
            }
        } else {
            Err(input.error("expected `= \"...\"`"))
        }
    }
}

/// Parse all `#[renderable(...)]` attributes from a `DeriveInput`.
fn parse_renderable_attrs(input: &DeriveInput) -> syn::Result<RenderableAttrs> {
    let mut attrs = RenderableAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("renderable") {
            continue;
        }
        let items: Punctuated<RenderableAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "via" => {
                    attrs.via = Some(renderable_expect_str(&item, "via")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown renderable attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn renderable_expect_str(attr: &RenderableAttr, _name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        RenderableAttrValue::Str(s) => Ok(s.clone()),
    }
}

// ---------------------------------------------------------------------------
// Renderable derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates a `gilt::console::Renderable` implementation for a struct.
///
/// This delegates rendering to one of the existing widget derives (Panel or Tree).
/// The struct must also derive the corresponding widget macro.
///
/// # Struct-level attributes (`#[renderable(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `via` | string | Widget to delegate to: `"panel"` (default) or `"tree"` |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::{Panel, Renderable};
///
/// #[derive(Panel, Renderable)]
/// #[renderable(via = "panel")]
/// #[panel(title = "Config", box_style = "ROUNDED")]
/// struct Config {
///     host: String,
///     port: u16,
/// }
///
/// // Config now implements gilt::console::Renderable
/// // and can be passed directly to console.print(&config)
/// ```
#[proc_macro_derive(Renderable, attributes(renderable))]
pub fn derive_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_renderable_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_renderable_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Only support structs (not enums or unions).
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Renderable derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Renderable derive does not support unions",
            ));
        }
    }

    // Parse struct-level #[renderable(...)] attributes.
    let renderable_attrs = parse_renderable_attrs(input)?;

    // Determine the delegation target. Default to "panel".
    let via = renderable_attrs
        .via
        .as_ref()
        .map(|lit| lit.value())
        .unwrap_or_else(|| "panel".to_string());

    let delegate_call = match via.as_str() {
        "panel" => {
            quote! { let widget = self.to_panel(); }
        }
        "tree" => {
            quote! { let widget = self.to_tree(); }
        }
        other => {
            let lit = renderable_attrs.via.as_ref().unwrap();
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "unknown renderable via `{}`. Expected one of: panel, tree",
                    other
                ),
            ));
        }
    };

    let expanded = quote! {
        impl gilt::console::Renderable for #struct_name {
            fn rich_console(
                &self,
                console: &gilt::console::Console,
                options: &gilt::console::ConsoleOptions,
            ) -> Vec<gilt::segment::Segment> {
                #delegate_call
                widget.rich_console(console, options)
            }
        }
    };

    Ok(expanded)
}

// ===========================================================================
// Columns derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[columns(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[columns(...)]` attributes.
#[derive(Default)]
struct ColumnsAttrs {
    column_count: Option<LitInt>,
    equal: Option<LitBool>,
    expand: Option<LitBool>,
    padding: Option<LitInt>,
    title: Option<LitStr>,
}

/// A single key=value (or standalone bool key) inside `#[columns(...)]`.
struct ColumnsAttr {
    key: Ident,
    value: ColumnsAttrValue,
}

enum ColumnsAttrValue {
    Str(LitStr),
    Bool(LitBool),
    Int(LitInt),
    /// Standalone flag like `expand` (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for ColumnsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(ColumnsAttr {
                    key,
                    value: ColumnsAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(ColumnsAttr {
                    key,
                    value: ColumnsAttrValue::Bool(lit),
                })
            } else if input.peek(LitInt) {
                let lit: LitInt = input.parse()?;
                Ok(ColumnsAttr {
                    key,
                    value: ColumnsAttrValue::Int(lit),
                })
            } else {
                Err(input.error("expected string literal, bool, or integer"))
            }
        } else {
            // Standalone flag
            Ok(ColumnsAttr {
                key,
                value: ColumnsAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[columns(...)]` attributes from a `DeriveInput`.
fn parse_columns_attrs(input: &DeriveInput) -> syn::Result<ColumnsAttrs> {
    let mut attrs = ColumnsAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("columns") {
            continue;
        }
        let items: Punctuated<ColumnsAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "column_count" => {
                    attrs.column_count = Some(columns_expect_int(&item, "column_count")?);
                }
                "equal" => {
                    attrs.equal = Some(columns_expect_bool(&item, "equal")?);
                }
                "expand" => {
                    attrs.expand = Some(columns_expect_bool(&item, "expand")?);
                }
                "padding" => {
                    attrs.padding = Some(columns_expect_int(&item, "padding")?);
                }
                "title" => {
                    attrs.title = Some(columns_expect_str(&item, "title")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown columns attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn columns_expect_str(attr: &ColumnsAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        ColumnsAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn columns_expect_bool(attr: &ColumnsAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        ColumnsAttrValue::Bool(b) => Ok(b.clone()),
        ColumnsAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

fn columns_expect_int(attr: &ColumnsAttr, name: &str) -> syn::Result<LitInt> {
    match &attr.value {
        ColumnsAttrValue::Int(i) => Ok(i.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects an integer literal", name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Columns derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates `to_card(&self) -> gilt::panel::Panel` and
/// `to_columns(items: &[Self]) -> gilt::columns::Columns` methods.
///
/// # Struct-level attributes (`#[columns(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `column_count` | int | Fixed number of columns (auto-detect if omitted) |
/// | `equal` | bool | Use equal-width columns |
/// | `expand` | bool | Expand to fill available width |
/// | `padding` | int | Horizontal padding between columns |
/// | `title` | string | Title displayed above the columns |
///
/// # Field-level attributes (`#[field(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `label` | string | Custom field label (default: Title Case field name) |
/// | `style` | string | Style applied as markup around the label |
/// | `skip` | bool | Exclude field from card |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Columns;
///
/// #[derive(Columns)]
/// #[columns(column_count = 3, equal = true, expand = true, padding = 2)]
/// struct ProjectCard {
///     #[field(label = "Project", style = "bold cyan")]
///     name: String,
///     #[field(label = "Status")]
///     status: String,
///     #[field(style = "dim")]
///     description: String,
///     #[field(skip)]
///     internal_id: u64,
/// }
///
/// let items = vec![
///     ProjectCard {
///         name: "Alpha".into(),
///         status: "Active".into(),
///         description: "First project".into(),
///         internal_id: 1,
///     },
/// ];
/// let cols = ProjectCard::to_columns(&items);
/// ```
#[proc_macro_derive(Columns, attributes(columns, field))]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_columns_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_columns_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Only support structs with named fields.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Columns derive only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Columns derive does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Columns derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Columns derive does not support unions",
            ));
        }
    };

    // Parse struct-level #[columns(...)] attributes.
    let columns_attrs = parse_columns_attrs(input)?;

    // Collect field info, respecting `skip`. Reuse FieldAttrs / parse_field_attrs.
    struct ColFieldInfo {
        ident: Ident,
        label: String,
        style: Option<String>,
    }
    let mut field_infos: Vec<ColFieldInfo> = Vec::new();

    for field in fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .clone();
        let fa = parse_field_attrs(field)?;

        // Check skip.
        let skip = fa.skip.as_ref().map(|b| b.value).unwrap_or(false);
        if skip {
            continue;
        }

        let label = match &fa.label {
            Some(lit) => lit.value(),
            None => snake_to_title_case(&ident.to_string()),
        };

        let style = fa.style.as_ref().map(|lit| lit.value());

        field_infos.push(ColFieldInfo {
            ident,
            label,
            style,
        });
    }

    // Build the line push expressions for each field (same pattern as Panel derive).
    let line_pushes: Vec<proc_macro2::TokenStream> = field_infos
        .iter()
        .map(|fi| {
            let ident = &fi.ident;
            let label = &fi.label;
            match &fi.style {
                Some(sty) => {
                    let open_tag = format!("[{}]", sty);
                    let close_tag = format!("[/{}]", sty);
                    quote! {
                        lines.push(format!("{}{}:{} {}", #open_tag, #label, #close_tag, self.#ident));
                    }
                }
                None => {
                    quote! {
                        lines.push(format!("{}: {}", #label, self.#ident));
                    }
                }
            }
        })
        .collect();

    // Build columns-level configuration statements.
    let mut cols_config = Vec::new();

    if let Some(ref lit) = columns_attrs.column_count {
        let val: usize = lit.base10_parse()?;
        cols_config.push(quote! {
            cols.width = Some(max_width / #val);
        });
    }
    if let Some(ref lit) = columns_attrs.equal {
        let val = lit.value;
        cols_config.push(quote! {
            cols.equal = #val;
        });
    }
    if let Some(ref lit) = columns_attrs.expand {
        let val = lit.value;
        cols_config.push(quote! {
            cols.expand = #val;
        });
    }
    if let Some(ref lit) = columns_attrs.padding {
        let val: usize = lit.base10_parse()?;
        cols_config.push(quote! {
            cols.padding = (0, #val, 0, #val);
        });
    }
    if let Some(ref lit) = columns_attrs.title {
        let val = lit.value();
        cols_config.push(quote! {
            cols.title = Some(#val.to_string());
        });
    }

    // Card title defaults to the struct name.
    let card_title = struct_name_str;

    let expanded = quote! {
        impl #struct_name {
            /// Renders this struct as a card (a Panel with labeled key-value fields).
            ///
            /// Each non-skipped field becomes a line `"Label: value"`. Field styles
            /// are applied as markup tags around the label.
            pub fn to_card(&self) -> gilt::panel::Panel {
                let mut lines: Vec<String> = Vec::new();
                #(#line_pushes)*
                let content = gilt::text::Text::from_markup(&lines.join("\n"))
                    .unwrap_or_else(|_| gilt::text::Text::from(lines.join("\n").as_str()));
                let mut panel = gilt::panel::Panel::new(content);
                panel.title = Some(gilt::text::Text::from(#card_title));
                panel
            }

            /// Creates a [`gilt::columns::Columns`] from a slice of items.
            ///
            /// Each item is rendered as a Panel card and laid out in columns.
            /// Struct-level `#[columns(...)]` attributes control the column layout.
            pub fn to_columns(items: &[Self]) -> gilt::columns::Columns {
                let mut cols = gilt::columns::Columns::new();
                #[allow(unused_variables)]
                let max_width: usize = 80;
                #(#cols_config)*
                for item in items {
                    let card = item.to_card();
                    cols.add_renderable(&format!("{}", card));
                }
                cols
            }
        }
    };

    Ok(expanded)
}

// ---------------------------------------------------------------------------
// Rule derive — attribute types
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[rule(...)]` attributes.
#[derive(Default)]
struct RuleAttrs {
    /// Custom title text (literal string). Overridden if a field has `#[rule(title)]`.
    title: Option<LitStr>,
    /// Character(s) used to draw the rule line (default "━").
    characters: Option<LitStr>,
    /// Style string for the rule line.
    style: Option<LitStr>,
    /// Title alignment: "left", "center", "right".
    align: Option<LitStr>,
    /// End string appended after the rule (default "\n").
    end: Option<LitStr>,
}

/// A single key=value inside `#[rule(...)]` at the struct level.
struct RuleAttr {
    key: Ident,
    value: RuleAttrValue,
}

enum RuleAttrValue {
    Str(LitStr),
}

impl Parse for RuleAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(RuleAttr {
                    key,
                    value: RuleAttrValue::Str(lit),
                })
            } else {
                Err(input.error("expected string literal"))
            }
        } else {
            Err(input.error("expected `= \"...\"`"))
        }
    }
}

/// Parse all `#[rule(...)]` attributes from a `DeriveInput`.
fn parse_rule_attrs(input: &DeriveInput) -> syn::Result<RuleAttrs> {
    let mut attrs = RuleAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("rule") {
            continue;
        }
        let items: Punctuated<RuleAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "title" => {
                    attrs.title = Some(rule_expect_str(&item, "title")?);
                }
                "characters" => {
                    attrs.characters = Some(rule_expect_str(&item, "characters")?);
                }
                "style" => {
                    attrs.style = Some(rule_expect_str(&item, "style")?);
                }
                "align" => {
                    attrs.align = Some(rule_expect_str(&item, "align")?);
                }
                "end" => {
                    attrs.end = Some(rule_expect_str(&item, "end")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown rule attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn rule_expect_str(attr: &RuleAttr, _name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        RuleAttrValue::Str(s) => Ok(s.clone()),
    }
}

/// Map an `align` string literal to a token stream for `gilt::align_widget::HorizontalAlign`.
fn align_tokens(lit: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    let val = lit.value();
    match val.as_str() {
        "left" => Ok(quote! { gilt::align_widget::HorizontalAlign::Left }),
        "center" => Ok(quote! { gilt::align_widget::HorizontalAlign::Center }),
        "right" => Ok(quote! { gilt::align_widget::HorizontalAlign::Right }),
        other => Err(syn::Error::new_spanned(
            lit,
            format!("unknown align `{other}`. Expected one of: left, center, right"),
        )),
    }
}

/// Check whether a field has `#[rule(title)]`.
fn has_rule_title_attr(field: &syn::Field) -> syn::Result<bool> {
    for attr in &field.attrs {
        if !attr.path().is_ident("rule") {
            continue;
        }
        let ident: Ident = attr.parse_args()?;
        if ident == "title" {
            return Ok(true);
        }
        return Err(syn::Error::new_spanned(
            &ident,
            format!("unknown rule field attribute `{}`. Expected: title", ident),
        ));
    }
    Ok(false)
}

// ---------------------------------------------------------------------------
// Rule derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates a `to_rule(&self) -> gilt::rule::Rule` method.
///
/// # Struct-level attributes (`#[rule(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `title` | string | Custom title text (default: struct name) |
/// | `characters` | string | Character(s) for the rule line (default "━") |
/// | `style` | string | Style string for the rule line |
/// | `align` | string | Title alignment: "left", "center", "right" |
/// | `end` | string | String appended after the rule (default "\n") |
///
/// # Field-level attributes (`#[rule(...)]`)
///
/// | Attribute | Description |
/// |-----------|-------------|
/// | `title` | Use this field's `.to_string()` as the rule title |
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Rule;
///
/// #[derive(Rule)]
/// #[rule(characters = "─", style = "bold blue", align = "center")]
/// struct SectionBreak {
///     #[rule(title)]
///     heading: String,
/// }
///
/// let br = SectionBreak { heading: "Results".into() };
/// let rule = br.to_rule();
/// ```
#[proc_macro_derive(Rule, attributes(rule))]
pub fn derive_rule(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_rule_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_rule_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Only support structs with named fields.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Rule derive only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Rule derive does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Rule derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Rule derive does not support unions",
            ));
        }
    };

    // Parse struct-level #[rule(...)] attributes.
    let rule_attrs = parse_rule_attrs(input)?;

    // Find the field annotated with `#[rule(title)]`, if any.
    let mut title_field: Option<Ident> = None;
    for field in fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .expect("named field must have ident")
            .clone();
        if has_rule_title_attr(field)? {
            if title_field.is_some() {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "only one field may be annotated with `#[rule(title)]`",
                ));
            }
            title_field = Some(ident);
        }
    }

    // Determine the title source.
    // Priority: field with #[rule(title)] > struct-level title attr > struct name.
    let title_expr = if let Some(ref field_ident) = title_field {
        quote! { self.#field_ident.to_string() }
    } else if let Some(ref lit) = rule_attrs.title {
        let val = lit.value();
        quote! { #val.to_string() }
    } else {
        quote! { #struct_name_str.to_string() }
    };

    // Build configuration statements.
    let mut rule_config = Vec::new();

    if let Some(ref lit) = rule_attrs.characters {
        let val = lit.value();
        rule_config.push(quote! {
            rule = rule.characters(#val);
        });
    }
    if let Some(ref lit) = rule_attrs.style {
        let val = lit.value();
        rule_config.push(quote! {
            rule = rule.style(gilt::style::Style::parse(#val).unwrap_or_else(|_| gilt::style::Style::null()));
        });
    }
    if let Some(ref lit) = rule_attrs.align {
        let align_ts = align_tokens(lit)?;
        rule_config.push(quote! {
            rule = rule.align(#align_ts);
        });
    }
    if let Some(ref lit) = rule_attrs.end {
        let val = lit.value();
        rule_config.push(quote! {
            rule = rule.end(#val);
        });
    }

    let expanded = quote! {
        impl #struct_name {
            /// Generates a [`gilt::rule::Rule`] from this struct.
            ///
            /// The title is derived from the field annotated with `#[rule(title)]`,
            /// the struct-level `title` attribute, or the struct name (in that order).
            pub fn to_rule(&self) -> gilt::rule::Rule {
                let title_text = #title_expr;
                let mut rule = gilt::rule::Rule::with_title(&title_text);
                #(#rule_config)*
                rule
            }
        }
    };

    Ok(expanded)
}

// ===========================================================================
// Inspect derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[inspect(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[inspect(...)]` attributes.
#[derive(Default)]
struct InspectAttrs {
    title: Option<LitStr>,
    label: Option<LitStr>,
    doc: Option<LitStr>,
    pretty: Option<LitBool>,
}

/// A single key=value (or standalone bool key) inside `#[inspect(...)]`.
struct InspectAttr {
    key: Ident,
    value: InspectAttrValue,
}

enum InspectAttrValue {
    Str(LitStr),
    Bool(LitBool),
    /// Standalone flag (no `= ...`), treated as `true`.
    Flag,
}

impl Parse for InspectAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Ok(InspectAttr {
                    key,
                    value: InspectAttrValue::Str(lit),
                })
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                Ok(InspectAttr {
                    key,
                    value: InspectAttrValue::Bool(lit),
                })
            } else {
                Err(input.error("expected string literal or bool"))
            }
        } else {
            // Standalone flag
            Ok(InspectAttr {
                key,
                value: InspectAttrValue::Flag,
            })
        }
    }
}

/// Parse all `#[inspect(...)]` attributes from a `DeriveInput`.
fn parse_inspect_attrs(input: &DeriveInput) -> syn::Result<InspectAttrs> {
    let mut attrs = InspectAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("inspect") {
            continue;
        }
        let items: Punctuated<InspectAttr, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;

        for item in items {
            let key_str = item.key.to_string();
            match key_str.as_str() {
                "title" => {
                    attrs.title = Some(inspect_expect_str(&item, "title")?);
                }
                "label" => {
                    attrs.label = Some(inspect_expect_str(&item, "label")?);
                }
                "doc" => {
                    attrs.doc = Some(inspect_expect_str(&item, "doc")?);
                }
                "pretty" => {
                    attrs.pretty = Some(inspect_expect_bool(&item, "pretty")?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &item.key,
                        format!("unknown inspect attribute `{}`", key_str),
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

fn inspect_expect_str(attr: &InspectAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        InspectAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

fn inspect_expect_bool(attr: &InspectAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        InspectAttrValue::Bool(b) => Ok(b.clone()),
        InspectAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

// ---------------------------------------------------------------------------
// Inspect derive entry point
// ---------------------------------------------------------------------------

/// Derive macro that generates a `to_inspect(&self) -> gilt::inspect::Inspect`
/// method on structs that implement `Debug`.
///
/// The generated method creates an [`Inspect`] widget for the value, applying
/// any struct-level `#[inspect(...)]` configuration attributes.
///
/// # Struct-level attributes (`#[inspect(...)]`)
///
/// | Attribute | Type | Description |
/// |-----------|------|-------------|
/// | `title` | string | Custom title for the inspect panel (default: "Inspect: TypeName") |
/// | `label` | string | Label for the inspected value |
/// | `doc` | string | Documentation text to display |
/// | `pretty` | bool | Pretty-print the Debug output (default true) |
///
/// # Requirements
///
/// The struct must implement `Debug` (or derive it). The generated impl adds
/// a `where Self: std::fmt::Debug + 'static` bound.
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Inspect;
///
/// #[derive(Debug, Inspect)]
/// #[inspect(title = "Server Info", label = "web-01")]
/// struct ServerStatus {
///     host: String,
///     cpu: f32,
///     memory: f32,
/// }
///
/// let status = ServerStatus {
///     host: "web-01".into(),
///     cpu: 42.5,
///     memory: 67.3,
/// };
/// let widget = status.to_inspect();
/// ```
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_inspect_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive_inspect_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Only support structs (not enums or unions).
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Inspect derive does not support enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Inspect derive does not support unions",
            ));
        }
    }

    // Parse struct-level #[inspect(...)] attributes.
    let inspect_attrs = parse_inspect_attrs(input)?;

    // Build configuration chain calls.
    let mut config_calls = Vec::new();

    if let Some(ref lit) = inspect_attrs.title {
        let val = lit.value();
        config_calls.push(quote! {
            .with_title(#val)
        });
    }
    if let Some(ref lit) = inspect_attrs.label {
        let val = lit.value();
        config_calls.push(quote! {
            .with_label(#val)
        });
    }
    if let Some(ref lit) = inspect_attrs.doc {
        let val = lit.value();
        config_calls.push(quote! {
            .with_doc(#val)
        });
    }
    if let Some(ref lit) = inspect_attrs.pretty {
        let val = lit.value;
        config_calls.push(quote! {
            .with_pretty(#val)
        });
    }

    let expanded = quote! {
        impl #struct_name {
            /// Creates a [`gilt::inspect::Inspect`] widget for this value.
            ///
            /// The struct must implement `Debug`. The inspect widget displays the
            /// type name, optional label/documentation, and the Debug representation
            /// with syntax highlighting.
            pub fn to_inspect(&self) -> gilt::inspect::Inspect<'_>
            where
                Self: std::fmt::Debug + 'static,
            {
                gilt::inspect::Inspect::new(self)
                    #(#config_calls)*
            }
        }
    };

    Ok(expanded)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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

    // -- TableAttr parsing -------------------------------------------------

    #[test]
    fn test_parse_table_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Title" };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "title");
        match attr.value {
            TableAttrValue::Str(s) => assert_eq!(s.value(), "My Title"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_table_attr_bool() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = true };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "expand");
        match attr.value {
            TableAttrValue::Bool(b) => assert!(b.value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_parse_table_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "expand");
        matches!(attr.value, TableAttrValue::Flag);
    }

    // -- ColumnAttr parsing ------------------------------------------------

    #[test]
    fn test_parse_column_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { header = "Name" };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "header");
        match attr.value {
            ColumnAttrValue::Str(s) => assert_eq!(s.value(), "Name"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_column_attr_int() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = 42 };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "width");
        match attr.value {
            ColumnAttrValue::Int(i) => assert_eq!(i.base10_parse::<usize>().unwrap(), 42),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn test_parse_column_attr_bool() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { no_wrap = true };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "no_wrap");
        match attr.value {
            ColumnAttrValue::Bool(b) => assert!(b.value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_parse_column_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "skip");
        matches!(attr.value, ColumnAttrValue::Flag);
    }

    // -- expect helpers ----------------------------------------------------

    #[test]
    fn test_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        let result = expect_str(&attr, "title");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "hello");
    }

    #[test]
    fn test_expect_str_wrong_type() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        let result = expect_str(&attr, "title");
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_bool_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = false };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        let result = expect_bool(&attr, "expand");
        assert!(result.is_ok());
        assert!(!result.unwrap().value);
    }

    #[test]
    fn test_expect_bool_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
        let attr: TableAttr = syn::parse2(tokens).unwrap();
        let result = expect_bool(&attr, "expand");
        assert!(result.is_ok());
        assert!(result.unwrap().value);
    }

    #[test]
    fn test_col_expect_int_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = 10 };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        let result = col_expect_int(&attr, "width");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().base10_parse::<usize>().unwrap(), 10);
    }

    #[test]
    fn test_col_expect_int_wrong_type() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { width = "ten" };
        let attr: ColumnAttr = syn::parse2(tokens).unwrap();
        let result = col_expect_int(&attr, "width");
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_table_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
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
        let result = derive_panel_impl(&input);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(
            tokens.contains("bold cyan"),
            "should apply title_style as markup"
        );
        assert!(tokens.contains("\"Info\""), "should contain title text");
    }

    // -- PanelAttr parsing -------------------------------------------------

    #[test]
    fn test_parse_panel_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Panel" };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "title");
        match attr.value {
            PanelAttrValue::Str(s) => assert_eq!(s.value(), "My Panel"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_panel_attr_bool() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand = false };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "expand");
        match attr.value {
            PanelAttrValue::Bool(b) => assert!(!b.value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_parse_panel_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { highlight };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "highlight");
        matches!(attr.value, PanelAttrValue::Flag);
    }

    // -- FieldAttr parsing -------------------------------------------------

    #[test]
    fn test_parse_field_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { label = "Host" };
        let attr: FieldAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "label");
        match attr.value {
            FieldAttrValue::Str(s) => assert_eq!(s.value(), "Host"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_field_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
        let attr: FieldAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "skip");
        matches!(attr.value, FieldAttrValue::Flag);
    }

    #[test]
    fn test_panel_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        let result = panel_expect_str(&attr, "title");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "hello");
    }

    #[test]
    fn test_panel_expect_str_wrong_type() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        let result = panel_expect_str(&attr, "title");
        assert!(result.is_err());
    }

    #[test]
    fn test_panel_expect_bool_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
        let attr: PanelAttr = syn::parse2(tokens).unwrap();
        let result = panel_expect_bool(&attr, "expand");
        assert!(result.is_ok());
        assert!(result.unwrap().value);
    }

    #[test]
    fn test_field_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { label = "Name" };
        let attr: FieldAttr = syn::parse2(tokens).unwrap();
        let result = field_expect_str(&attr, "label");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "Name");
    }

    #[test]
    fn test_field_expect_bool_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { skip };
        let attr: FieldAttr = syn::parse2(tokens).unwrap();
        let result = field_expect_bool(&attr, "skip");
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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
        let result = derive_tree_impl(&input);
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

    // -- TreeAttr parsing --------------------------------------------------

    #[test]
    fn test_parse_tree_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { style = "bold" };
        let attr: TreeAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "style");
        match attr.value {
            TreeAttrValue::Str(s) => assert_eq!(s.value(), "bold"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_tree_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { style };
        let attr: TreeAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "style");
        matches!(attr.value, TreeAttrValue::Flag);
    }

    #[test]
    fn test_tree_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { guide_style = "dim cyan" };
        let attr: TreeAttr = syn::parse2(tokens).unwrap();
        let result = tree_expect_str(&attr, "guide_style");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "dim cyan");
    }

    #[test]
    fn test_tree_expect_str_wrong_type() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { style };
        let attr: TreeAttr = syn::parse2(tokens).unwrap();
        let result = tree_expect_str(&attr, "style");
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
        let result = derive_renderable_impl(&input);
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
            tokens.contains("rich_console"),
            "should generate rich_console method"
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
        let result = derive_renderable_impl(&input);
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
            tokens.contains("rich_console"),
            "should generate rich_console method"
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
        let result = derive_renderable_impl(&input);
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
        let result = derive_renderable_impl(&input);
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
        let result = derive_renderable_impl(&input);
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
        let result = derive_renderable_impl(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown renderable attribute"),);
    }

    // -- RenderableAttr parsing --------------------------------------------

    #[test]
    fn test_parse_renderable_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { via = "panel" };
        let attr: RenderableAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "via");
        match attr.value {
            RenderableAttrValue::Str(s) => assert_eq!(s.value(), "panel"),
        }
    }

    #[test]
    fn test_renderable_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { via = "tree" };
        let attr: RenderableAttr = syn::parse2(tokens).unwrap();
        let result = renderable_expect_str(&attr, "via");
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
        let result = derive_columns_impl(&input);
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
        let result = derive_columns_impl(&input);
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
        let result = derive_columns_impl(&input);
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
        let result = derive_columns_impl(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support enums"));
    }

    // -- ColumnsAttr parsing -----------------------------------------------

    #[test]
    fn test_parse_columns_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Cols" };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "title");
        match attr.value {
            ColumnsAttrValue::Str(s) => assert_eq!(s.value(), "My Cols"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_columns_attr_bool() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { equal = true };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "equal");
        match attr.value {
            ColumnsAttrValue::Bool(b) => assert!(b.value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_parse_columns_attr_int() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { column_count = 4 };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "column_count");
        match attr.value {
            ColumnsAttrValue::Int(i) => assert_eq!(i.base10_parse::<usize>().unwrap(), 4),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn test_parse_columns_attr_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "expand");
        matches!(attr.value, ColumnsAttrValue::Flag);
    }

    #[test]
    fn test_columns_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        let result = columns_expect_str(&attr, "title");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "hello");
    }

    #[test]
    fn test_columns_expect_bool_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { expand };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        let result = columns_expect_bool(&attr, "expand");
        assert!(result.is_ok());
        assert!(result.unwrap().value);
    }

    #[test]
    fn test_columns_expect_int_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { padding = 3 };
        let attr: ColumnsAttr = syn::parse2(tokens).unwrap();
        let result = columns_expect_int(&attr, "padding");
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
        let result = derive_columns_impl(&input);
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
        let result = derive_columns_impl(&input);
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
        let result = derive_rule_impl(&input);
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
        let result = derive_rule_impl(&input);
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
        let result = derive_rule_impl(&input);
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
        let result = derive_rule_impl(&input);
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
        let result = derive_rule_impl(&input);
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
        let result = derive_rule_impl(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support enums"));
    }

    // -- RuleAttr parsing --------------------------------------------------

    #[test]
    fn test_parse_rule_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { style = "bold red" };
        let attr: RuleAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "style");
        match attr.value {
            RuleAttrValue::Str(s) => assert_eq!(s.value(), "bold red"),
        }
    }

    #[test]
    fn test_rule_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { characters = "─" };
        let attr: RuleAttr = syn::parse2(tokens).unwrap();
        let result = rule_expect_str(&attr, "characters");
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
        let result = derive_inspect_impl(&input);
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
        let result = derive_inspect_impl(&input);
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
        let result = derive_inspect_impl(&input);
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
        let result = derive_inspect_impl(&input);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("with_pretty"), "should call with_pretty");
    }

    #[test]
    fn test_derive_inspect_rejects_enum() {
        let input: DeriveInput = syn::parse_quote! {
            enum Foo { A, B }
        };
        let result = derive_inspect_impl(&input);
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
        let result = derive_inspect_impl(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown inspect attribute"),);
    }

    // -- InspectAttr parsing -----------------------------------------------

    #[test]
    fn test_parse_inspect_attr_str() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "My Inspect" };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "title");
        match attr.value {
            InspectAttrValue::Str(s) => assert_eq!(s.value(), "My Inspect"),
            _ => panic!("expected Str"),
        }
    }

    #[test]
    fn test_parse_inspect_attr_bool() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty = false };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        assert_eq!(attr.key, "pretty");
        match attr.value {
            InspectAttrValue::Bool(b) => assert!(!b.value),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_inspect_expect_str_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = "hello" };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        let result = inspect_expect_str(&attr, "title");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "hello");
    }

    #[test]
    fn test_inspect_expect_str_wrong_type() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { title = true };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        let result = inspect_expect_str(&attr, "title");
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_expect_bool_flag() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        let result = inspect_expect_bool(&attr, "pretty");
        assert!(result.is_ok());
        assert!(result.unwrap().value);
    }

    #[test]
    fn test_inspect_expect_bool_ok() {
        let tokens: proc_macro2::TokenStream = syn::parse_quote! { pretty = false };
        let attr: InspectAttr = syn::parse2(tokens).unwrap();
        let result = inspect_expect_bool(&attr, "pretty");
        assert!(result.is_ok());
        assert!(!result.unwrap().value);
    }
}
