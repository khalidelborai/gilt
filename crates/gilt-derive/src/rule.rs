//! `Rule` derive macro implementation.
//!
//! Generates a `to_rule(&self) -> gilt::rule::Rule` method.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, Token};

use crate::shared::named_field_ident;

// ---------------------------------------------------------------------------
// Rule derive — attribute types
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[rule(...)]` attributes.
#[derive(Default)]
pub(crate) struct RuleAttrs {
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
pub(crate) struct RuleAttr {
    pub(crate) key: Ident,
    pub(crate) value: RuleAttrValue,
}

pub(crate) enum RuleAttrValue {
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
pub(crate) fn parse_rule_attrs(input: &DeriveInput) -> syn::Result<RuleAttrs> {
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

pub(crate) fn rule_expect_str(attr: &RuleAttr, _name: &str) -> syn::Result<LitStr> {
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
pub(crate) fn has_rule_title_attr(field: &syn::Field) -> syn::Result<bool> {
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

pub(crate) fn derive_rule_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
        let ident = named_field_ident(field)?.clone();
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
            rule = rule.with_characters(#val);
        });
    }
    if let Some(ref lit) = rule_attrs.style {
        let val = lit.value();
        rule_config.push(quote! {
            rule = rule.with_style(gilt::style::Style::parse(#val).unwrap_or_else(|_| gilt::style::Style::null()));
        });
    }
    if let Some(ref lit) = rule_attrs.align {
        let align_ts = align_tokens(lit)?;
        rule_config.push(quote! {
            rule = rule.with_align(#align_ts);
        });
    }
    if let Some(ref lit) = rule_attrs.end {
        let val = lit.value();
        rule_config.push(quote! {
            rule = rule.with_end(#val);
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
