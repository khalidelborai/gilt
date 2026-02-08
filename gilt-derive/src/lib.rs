//! Derive macros for the gilt terminal formatting library.
//!
//! This crate provides the `#[derive(Table)]` macro that generates a
//! `to_table()` method for structs, converting struct fields into table columns.
//!
//! # Example
//!
//! ```ignore
//! use gilt::Table;
//!
//! #[derive(Table)]
//! struct Employee {
//!     name: String,
//!     age: u32,
//!     department: String,
//! }
//!
//! let employees = vec![
//!     Employee { name: "Alice".into(), age: 30, department: "Engineering".into() },
//!     Employee { name: "Bob".into(), age: 25, department: "Marketing".into() },
//! ];
//! let table = Employee::to_table(&employees);
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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

/// Derive macro that generates a `to_table(items: &[Self]) -> gilt::table::Table` method.
///
/// Each struct field becomes a column. The column header is the field name
/// converted to Title Case (e.g., `first_name` -> "First Name").
///
/// All fields must implement `std::fmt::Display`.
///
/// # Example
///
/// ```ignore
/// use gilt_derive::Table;
///
/// #[derive(Table)]
/// struct Employee {
///     name: String,
///     age: u32,
///     department: String,
/// }
///
/// let employees = vec![
///     Employee { name: "Alice".into(), age: 30, department: "Engineering".into() },
///     Employee { name: "Bob".into(), age: 25, department: "Marketing".into() },
/// ];
/// let table = Employee::to_table(&employees);
/// ```
#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Table derive only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Table derive does not support unit structs",
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Table derive does not support enums",
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Table derive does not support unions",
            )
            .to_compile_error()
            .into();
        }
    };

    // Build header strings and field accessors
    let headers: Vec<String> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().expect("named field must have ident");
            snake_to_title_case(&name.to_string())
        })
        .collect();

    let field_idents: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("named field must have ident"))
        .collect();

    let header_literals: Vec<_> = headers.iter().map(|h| h.as_str()).collect::<Vec<_>>();
    let header_tokens = header_literals.iter().map(|h| quote! { #h });

    let row_expr = field_idents.iter().map(|ident| {
        quote! { &item.#ident.to_string() }
    });

    let expanded = quote! {
        impl #struct_name {
            /// Creates a [`gilt::table::Table`] from a slice of items.
            ///
            /// Each struct field becomes a column, with headers derived from
            /// field names converted to Title Case. The table title is set to
            /// the struct name.
            pub fn to_table(items: &[Self]) -> gilt::table::Table {
                let mut table = gilt::table::Table::new(&[#(#header_tokens),*]);
                table.title = Some(#struct_name_str.to_string());
                for item in items {
                    table.add_row(&[#(#row_expr),*]);
                }
                table
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::snake_to_title_case;

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
    }
}
