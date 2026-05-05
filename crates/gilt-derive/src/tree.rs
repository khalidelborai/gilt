//! `Tree` derive macro implementation.
//!
//! Generates a `to_tree(&self) -> gilt::tree::Tree` method.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Fields, Ident, LitStr, Token};

use crate::shared::named_field_ident;

// ===========================================================================
// Tree derive macro
// ===========================================================================

// ---------------------------------------------------------------------------
// Struct-level attribute: #[tree(...)]
// ---------------------------------------------------------------------------

/// Parsed struct-level `#[tree(...)]` attributes.
#[derive(Default)]
pub(crate) struct TreeAttrs {
    style: Option<LitStr>,
    guide_style: Option<LitStr>,
}

/// A single key=value inside `#[tree(...)]` at the struct level.
pub(crate) struct TreeAttr {
    pub(crate) key: Ident,
    pub(crate) value: TreeAttrValue,
}

pub(crate) enum TreeAttrValue {
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
pub(crate) fn parse_tree_attrs(input: &DeriveInput) -> syn::Result<TreeAttrs> {
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

pub(crate) fn tree_expect_str(attr: &TreeAttr, name: &str) -> syn::Result<LitStr> {
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
pub(crate) enum TreeFieldKind {
    Label,
    Children,
    Leaf,
    None,
}

/// Parse `#[tree(...)]` attributes on a field to determine its role.
pub(crate) fn parse_tree_field_attrs(field: &syn::Field) -> syn::Result<TreeFieldKind> {
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
pub(crate) fn derive_tree_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
        let ident = named_field_ident(field)?.clone();
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
            if let Ok(s) = gilt::style::Style::parse_strict(#val) {
                tree.style = s;
            }
        }
    } else {
        quote! {}
    };

    let guide_style_setup = if let Some(ref lit) = tree_attrs.guide_style {
        let val = lit.value();
        quote! {
            if let Ok(s) = gilt::style::Style::parse_strict(#val) {
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
            let leaf_label = crate::shared::snake_to_title_case(&ident.to_string());
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
