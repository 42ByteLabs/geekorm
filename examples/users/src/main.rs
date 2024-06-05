#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use geekorm::prelude::*;
use geekorm::{GeekTable, PrimaryKeyInteger};

#[derive(Debug, Clone, Default, GeekTable)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,
    username: String,
    email: String,
    active: bool,
    postcode: Option<String>,
}

fn main() {
    // Build a CREATE TABLE query
    let create_query = Users::create().build().expect("Failed to build query");

    println!("Create Query: {}", create_query);

    // Build a SELECT query with WHERE, OR and LIKE
    let select_query = Users::select()
        .where_eq("username", "geekmasher")
        .or()
        .where_like("email", "%geekmasher%")
        .build()
        .expect("Failed to build query");

    println!("Select Query : {}", select_query);
    println!("Select Values: {:?}", select_query.values);
}
