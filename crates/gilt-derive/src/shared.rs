//! Cross-cutting helpers shared by every derive in this crate.
//!
//! Kept together so that future per-derive modules don't pull duplicate
//! mappings or accidentally diverge on shared concepts (named-field
//! validation, snake-case formatting, the `box_style` enum mirror, the
//! `justify` enum mirror).

use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr};

/// Get a named field's ident.
///
/// Every gilt-derive macro iterates `Fields::Named` (struct with named
/// fields), so `field.ident` is logically always `Some`. This helper exists
/// so that — if a future code path ever passes through an unnamed field
/// (e.g. via macro composition or refactoring) — we surface a `compile_error!`
/// pointing at the offending field instead of panicking the proc-macro.
pub(crate) fn named_field_ident(field: &syn::Field) -> syn::Result<&syn::Ident> {
    field.ident.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            field,
            "expected named field — gilt-derive only supports structs with named fields",
        )
    })
}

/// Convert a snake_case identifier to Title Case.
///
/// Examples:
/// - `first_name` -> "First Name"
/// - `age` -> "Age"
/// - `department_id` -> "Department Id"
pub(crate) fn snake_to_title_case(s: &str) -> String {
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

/// Map a `box_style` string literal to a token stream referencing the
/// corresponding `gilt::box_chars::*` static.
pub(crate) fn box_style_tokens(lit: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
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

/// Map a `justify` string literal to a token stream for `gilt::text::JustifyMethod`.
pub(crate) fn justify_tokens(lit: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
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
