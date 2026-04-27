//! `Table` derive macro implementation.
//!
//! Generates a `to_table(items: &[Self]) -> gilt::table::Table` method.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Ident, LitBool, LitInt, LitStr, Token};

use crate::shared::{box_style_tokens, justify_tokens, named_field_ident, snake_to_title_case};

// ---------------------------------------------------------------------------
// Struct-level attribute: #[table(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[table(...)]` attributes.
#[derive(Default)]
pub(crate) struct TableAttrs {
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
pub(crate) struct TableAttr {
    pub(crate) key: Ident,
    pub(crate) value: TableAttrValue,
}

pub(crate) enum TableAttrValue {
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
pub(crate) fn parse_table_attrs(input: &DeriveInput) -> syn::Result<TableAttrs> {
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

pub(crate) fn expect_str(attr: &TableAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        TableAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn expect_bool(attr: &TableAttr, _name: &str) -> syn::Result<LitBool> {
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
pub(crate) struct ColumnAttrs {
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
pub(crate) struct ColumnAttr {
    pub(crate) key: Ident,
    pub(crate) value: ColumnAttrValue,
}

pub(crate) enum ColumnAttrValue {
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
pub(crate) fn parse_column_attrs(field: &syn::Field) -> syn::Result<ColumnAttrs> {
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

pub(crate) fn col_expect_str(attr: &ColumnAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        ColumnAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn col_expect_bool(attr: &ColumnAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        ColumnAttrValue::Bool(b) => Ok(b.clone()),
        ColumnAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

pub(crate) fn col_expect_int(attr: &ColumnAttr, name: &str) -> syn::Result<LitInt> {
    match &attr.value {
        ColumnAttrValue::Int(i) => Ok(i.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects an integer literal", name),
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

pub(crate) fn derive_table_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
        let ident = named_field_ident(field)?.clone();
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
