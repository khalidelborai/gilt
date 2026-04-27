//! `Renderable` derive macro implementation.
//!
//! Generates an `impl gilt::console::Renderable` for the target struct that
//! delegates rendering to one of the existing widget derives — currently
//! `Panel` (default) or `Tree`. Combined with `#[derive(Panel)]` or
//! `#[derive(Tree)]`, this lets a struct be passed directly to
//! `Console::print` and other places expecting a `Renderable`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Ident, LitStr, Token};

// ---------------------------------------------------------------------------
// Struct-level attribute: #[renderable(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[renderable(...)]` attributes.
#[derive(Default)]
pub(crate) struct RenderableAttrs {
    /// Which widget to delegate to: "panel" or "tree". Defaults to "panel".
    pub(crate) via: Option<LitStr>,
}

/// A single key=value (or standalone bool key) inside `#[renderable(...)]`.
pub(crate) struct RenderableAttr {
    pub(crate) key: Ident,
    pub(crate) value: RenderableAttrValue,
}

pub(crate) enum RenderableAttrValue {
    Str(LitStr),
}

impl Parse for RenderableAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
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
    }
}

/// Parse all `#[renderable(...)]` attributes from a `DeriveInput`.
pub(crate) fn parse_renderable_attrs(input: &DeriveInput) -> syn::Result<RenderableAttrs> {
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

pub(crate) fn renderable_expect_str(attr: &RenderableAttr, _name: &str) -> syn::Result<LitStr> {
    match &attr.value {
        RenderableAttrValue::Str(s) => Ok(s.clone()),
    }
}

// ---------------------------------------------------------------------------
// Renderable derive impl
// ---------------------------------------------------------------------------

pub(crate) fn derive_renderable_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
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

    // Determine the delegation target. Default to "panel" when `via` is absent.
    // Restructured to keep the `LitStr` (for spanned errors) in scope rather
    // than recovering it via `.unwrap()` after a value-only match — that
    // unwrap was logically safe but its safety lived in the control flow,
    // not in the types.
    let delegate_call = match renderable_attrs.via.as_ref() {
        None => quote! { let widget = self.to_panel(); },
        Some(lit) => match lit.value().as_str() {
            "panel" => quote! { let widget = self.to_panel(); },
            "tree" => quote! { let widget = self.to_tree(); },
            other => {
                return Err(syn::Error::new_spanned(
                    lit,
                    format!(
                        "unknown renderable via `{}`. Expected one of: panel, tree",
                        other
                    ),
                ));
            }
        },
    };

    let expanded = quote! {
        impl gilt::console::Renderable for #struct_name {
            fn gilt_console(
                &self,
                console: &gilt::console::Console,
                options: &gilt::console::ConsoleOptions,
            ) -> Vec<gilt::segment::Segment> {
                #delegate_call
                widget.gilt_console(console, options)
            }
        }
    };

    Ok(expanded)
}
