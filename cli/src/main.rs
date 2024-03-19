#![allow(unused_imports, dead_code, unused_variables)]
use anyhow::Result;
// Imports
use geekorm::prelude::*;
use geekorm_derive::GeekTable;

#[derive(Debug, Default, GeekTable)]
pub struct User {
    pub name: String,
    pub age: i32,
    pub postcode: Option<String>,
}

fn main() -> Result<()> {
    println!("Starting...\n");

    let table = User::table();
    println!("Table :: {:?}\n", table);

    let query = User::create().build()?;
    println!("Create :: `{}`\n", query);

    let select = User::select()
        .where_ne("name", "Mathew")
        // .and()
        .and()
        .where_gt("age", 20)
        .and()
        .where_lt("age", 30)
        // order by name asc
        .order_by("age", QueryOrder::Asc)
        // build
        .build()?;

    println!("Select :: `{}`", select);
    println!("Values :: {:?}\n", select.values);

    let count = User::count().build()?;
    println!("Count  :: `{}`", count);

    let limit = User::select()
        .order_by("age", QueryOrder::Asc)
        .limit(10)
        .offset(0)
        .build()?;
    println!("Limit  :: `{}`", limit);

    Ok(())
}
