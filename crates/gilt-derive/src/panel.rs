//! `Panel` derive macro implementation.
//!
//! Generates a `to_panel(&self) -> gilt::panel::Panel` method.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Ident, LitBool, LitStr, Token};

use crate::shared::{box_style_tokens, named_field_ident, snake_to_title_case};

// ===========================================================================
// Panel derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[panel(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[panel(...)]` attributes.
#[derive(Default)]
pub(crate) struct PanelAttrs {
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
pub(crate) struct PanelAttr {
    pub(crate) key: Ident,
    pub(crate) value: PanelAttrValue,
}

pub(crate) enum PanelAttrValue {
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
pub(crate) fn parse_panel_attrs(input: &DeriveInput) -> syn::Result<PanelAttrs> {
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

pub(crate) fn panel_expect_str(attr: &PanelAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        PanelAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn panel_expect_bool(attr: &PanelAttr, _name: &str) -> syn::Result<LitBool> {
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
pub(crate) struct FieldAttrs {
    pub(crate) label: Option<LitStr>,
    pub(crate) style: Option<LitStr>,
    pub(crate) skip: Option<LitBool>,
}

/// A single key=value (or standalone flag) inside `#[field(...)]`.
pub(crate) struct FieldAttr {
    pub(crate) key: Ident,
    pub(crate) value: FieldAttrValue,
}

pub(crate) enum FieldAttrValue {
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
pub(crate) fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
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

pub(crate) fn field_expect_str(attr: &FieldAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        FieldAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn field_expect_bool(attr: &FieldAttr, _name: &str) -> syn::Result<LitBool> {
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
pub(crate) fn derive_panel_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
        let ident = named_field_ident(field)?.clone();
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
            panel.border_style = gilt::style::Style::parse(#val);
        });
    }
    if let Some(ref lit) = panel_attrs.style {
        let val = lit.value();
        panel_config.push(quote! {
            panel.style = gilt::style::Style::parse(#val);
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
