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

mod columns;
mod inspect;
mod panel;
mod renderable;
mod rule;
mod shared;
mod table;
mod text_macro;
mod tree;

use proc_macro::TokenStream;
#[cfg(test)]
use proc_macro2::Span;
use syn::{parse_macro_input, DeriveInput};

// ===========================================================================
// Tree derive entry point (impl moved to crate::tree in v0.11.4)
// ===========================================================================

/// Derive macro generating `to_tree(&self) -> gilt::tree::Tree`.
/// See [`crate::tree`] for full attribute schema.
#[proc_macro_derive(Tree, attributes(tree))]
pub fn derive_tree(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match tree::derive_tree_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Renderable derive entry point (impl moved to crates::renderable in v0.11.4)
// ===========================================================================

/// Derive macro that generates a `gilt::console::Renderable` implementation
/// for a struct, delegating to one of the existing widget derives.
///
/// See [`crate::renderable`] for full attribute schema and behaviour.
#[proc_macro_derive(Renderable, attributes(renderable))]
pub fn derive_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match renderable::derive_renderable_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Inspect derive entry point (impl moved to crate::inspect in v0.11.4)
// ===========================================================================

/// Derive macro that generates a `to_inspect(&self) -> gilt::inspect::Inspect`
/// method for structs implementing `Debug`. See [`crate::inspect`] for the
/// full attribute schema.
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match inspect::derive_inspect_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Rule derive entry point (impl moved to crate::rule in v0.11.4)
// ===========================================================================

/// Derive macro that generates a `to_rule(&self) -> gilt::rule::Rule` method.
/// See [`crate::rule`] for the full attribute schema.
#[proc_macro_derive(Rule, attributes(rule))]
pub fn derive_rule(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match rule::derive_rule_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Columns derive entry point (impl moved to crate::columns in v0.11.4)
// ===========================================================================

/// Derive macro generating `to_columns(items: &[Self]) -> gilt::columns::Columns`.
/// See [`crate::columns`] for full attribute schema.
#[proc_macro_derive(Columns, attributes(columns, field))]
pub fn derive_columns(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match columns::derive_columns_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Panel derive entry point (impl moved to crate::panel in v0.11.4)
// ===========================================================================

/// Derive macro generating `to_panel(&self) -> gilt::panel::Panel`.
/// See [`crate::panel`] for full attribute schema.
#[proc_macro_derive(Panel, attributes(panel, field))]
pub fn derive_panel(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match panel::derive_panel_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// Table derive entry point (impl moved to crate::table in v0.11.4)
// ===========================================================================

/// Derive macro generating `to_table(items: &[Self]) -> gilt::table::Table`.
/// See [`crate::table`] for full attribute schema.
#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match table::derive_table_impl(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ===========================================================================
// text! function-like proc macro
// ===========================================================================

/// Validate gilt markup at compile time and expand to a `gilt::text::Text`.
///
/// Typos that `Text::from_markup` would silently accept (or panic on at
/// runtime) become `cargo build` errors:
///
/// ```ignore
/// use gilt::text;
///
/// // ✓ Compiles — markup is valid.
/// let t = text!("[bold red]Error:[/] file not found");
///
/// // ✗ Compile error — unknown token "blod".
/// // let bad = text!("[blod]text[/]");
/// ```
///
/// # Limitations
///
/// Accepts only **string literals** — no format-string interpolation.
/// For dynamic markup, use [`gilt::text::Text::from_markup`] directly.
///
/// # Note on unclosed tags
///
/// Following gilt's runtime behaviour, unclosed tags are **valid**: they apply
/// to the remainder of the text.  The macro does **not** error on `"[bold]hi"`.
#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as syn::LitStr);
    text_macro::text_macro_impl(lit).into()
}

// ---------------------------------------------------------------------------
// Tests (extracted to tests.rs in v1.2 cleanup)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
