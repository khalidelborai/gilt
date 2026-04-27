//! `Inspect` derive macro implementation.
//!
//! Generates a `to_inspect(&self) -> gilt::inspect::Inspect` method on
//! structs implementing `Debug`. The widget displays the type name,
//! optional label/documentation, and the Debug representation with
//! syntax highlighting.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Ident, LitBool, LitStr, Token};

// ---------------------------------------------------------------------------
// Struct-level attribute: #[inspect(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[inspect(...)]` attributes.
#[derive(Default)]
pub(crate) struct InspectAttrs {
    pub(crate) title: Option<LitStr>,
    pub(crate) label: Option<LitStr>,
    pub(crate) doc: Option<LitStr>,
    pub(crate) pretty: Option<LitBool>,
}

/// A single key=value (or standalone bool key) inside `#[inspect(...)]`.
pub(crate) struct InspectAttr {
    pub(crate) key: Ident,
    pub(crate) value: InspectAttrValue,
}

pub(crate) enum InspectAttrValue {
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
pub(crate) fn parse_inspect_attrs(input: &DeriveInput) -> syn::Result<InspectAttrs> {
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

pub(crate) fn inspect_expect_str(attr: &InspectAttr, name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        InspectAttrValue::Str(s) => Ok(s.clone()),
        _ => Err(syn::Error::new_spanned(
            &attr.key,
            format!("`{}` expects a string literal", name),
        )),
    }
}

pub(crate) fn inspect_expect_bool(attr: &InspectAttr, _name: &str) -> syn::Result<LitBool> {
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
// Inspect derive impl
// ---------------------------------------------------------------------------

pub(crate) fn derive_inspect_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
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
