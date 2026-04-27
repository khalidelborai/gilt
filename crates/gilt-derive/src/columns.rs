//! `Columns` derive macro implementation.
//!
//! Generates a `to_columns(items: &[Self]) -> gilt::columns::Columns` method.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Ident, LitBool, LitInt, LitStr, Token};

use crate::shared::named_field_ident;

// ===========================================================================
// Columns derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[columns(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[columns(...)]` attributes.
#[derive(Default)]
pub(crate) struct ColumnsAttrs {
    column_count: Option<LitInt>,
    equal: Option<LitBool>,
    expand: Option<LitBool>,
    padding: Option<LitInt>,
    title: Option<LitStr>,
}

/// A single key=value (or standalone bool key) inside `#[columns(...)]`.
pub(crate) struct ColumnsAttr {
    pub(crate) key: Ident,
    pub(crate) value: ColumnsAttrValue,
}

pub(crate) enum ColumnsAttrValue {
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
pub(crate) fn parse_columns_attrs(input: &DeriveInput) -> syn::Result<ColumnsAttrs> {
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

pub(crate) fn columns_expect_str(attr: &ColumnsAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        ColumnsAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn columns_expect_bool(attr: &ColumnsAttr, _name: &str) -> syn::Result<LitBool> {
    match &attr.value {
        ColumnsAttrValue::Bool(b) => Ok(b.clone()),
        ColumnsAttrValue::Flag => Ok(LitBool::new(true, attr.key.span())),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a bool", _name),
        )),
    }
}

pub(crate) fn columns_expect_int(attr: &ColumnsAttr, name: &str) -> syn::Result<LitInt> {
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
pub(crate) fn derive_columns_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

    // Collect field info, respecting `skip`. Reuse FieldAttrs / crate::panel::parse_field_attrs.
    struct ColFieldInfo {
        ident: Ident,
        label: String,
        style: Option<String>,
    }
    let mut field_infos: Vec<ColFieldInfo> = Vec::new();

    for field in fields.iter() {
        let ident = named_field_ident(field)?.clone();
        let fa = crate::panel::parse_field_attrs(field)?;

        // Check skip.
        let skip = fa.skip.as_ref().map(|b| b.value).unwrap_or(false);
        if skip {
            continue;
        }

        let label = match &fa.label {
            Some(lit) => lit.value(),
            None => crate::shared::snake_to_title_case(&ident.to_string()),
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
