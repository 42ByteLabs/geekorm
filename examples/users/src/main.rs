#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use geekorm::prelude::*;
use geekorm::{GeekTable, PrimaryKeyInteger};

#[derive(GeekValue, Debug, Clone, Default)]
enum UserType {
    Admin,
    #[default]
    User,
    Guest,
}

#[derive(Debug, Clone, Default, GeekTable, serde::Serialize, serde::Deserialize)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    username: String,

    #[geekorm(unique)]
    email: String,

    user_type: UserType,

    #[geekorm(new = false)]
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
