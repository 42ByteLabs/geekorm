#![allow(dead_code)]
use geekorm::prelude::*;
use geekorm::GeekTable;

#[derive(Debug, Clone, GeekTable)]
struct User {
    username: String,
    email: String,
    active: bool,
    postcode: Option<String>,
}

fn main() {
    // Build a CREATE TABLE query
    let create_query = User::create().build().expect("Failed to build query");

    println!("Create Query: {}", create_query);

    // Build a SELECT query with WHERE, OR and LIKE
    let select_query = User::select()
        .where_eq("username", "geekmasher")
        .or()
        .where_like("email", "%geekmasher%")
        .build()
        .expect("Failed to build query");

    println!("Select Query : {}", select_query);
    println!("Select Values: {:?}", select_query.values);
}
