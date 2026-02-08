#![cfg(feature = "derive")]

use gilt::Table;

#[derive(Table)]
struct Movie {
    title: String,
    year: u32,
    rating: f64,
}

#[test]
fn test_derive_table_creates_table() {
    let movies = vec![
        Movie {
            title: "Inception".into(),
            year: 2010,
            rating: 8.8,
        },
        Movie {
            title: "The Matrix".into(),
            year: 1999,
            rating: 8.7,
        },
    ];
    let table = Movie::to_table(&movies);
    assert_eq!(table.columns.len(), 3);
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.title.as_deref(), Some("Movie"));
}

#[derive(Table)]
struct Simple {
    name: String,
    value: i32,
}

#[test]
fn test_derive_table_simple() {
    let items = vec![Simple {
        name: "a".into(),
        value: 1,
    }];
    let table = Simple::to_table(&items);
    assert_eq!(table.columns.len(), 2);
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.title.as_deref(), Some("Simple"));
}

#[test]
fn test_derive_table_empty_vec() {
    let items: Vec<Simple> = vec![];
    let table = Simple::to_table(&items);
    assert_eq!(table.columns.len(), 2);
    assert_eq!(table.rows.len(), 0);
}

#[derive(Table)]
struct SingleField {
    id: u64,
}

#[test]
fn test_derive_table_single_field() {
    let items = vec![SingleField { id: 42 }];
    let table = SingleField::to_table(&items);
    assert_eq!(table.columns.len(), 1);
    assert_eq!(table.rows.len(), 1);
}

#[derive(Table)]
struct SnakeCaseFields {
    first_name: String,
    last_name: String,
    employee_id: u32,
}

#[test]
fn test_derive_table_snake_case_headers() {
    let items = vec![SnakeCaseFields {
        first_name: "Alice".into(),
        last_name: "Smith".into(),
        employee_id: 101,
    }];
    let table = SnakeCaseFields::to_table(&items);
    assert_eq!(table.columns.len(), 3);
    // Verify headers are Title Case
    assert_eq!(table.columns[0].header, "First Name");
    assert_eq!(table.columns[1].header, "Last Name");
    assert_eq!(table.columns[2].header, "Employee Id");
}
