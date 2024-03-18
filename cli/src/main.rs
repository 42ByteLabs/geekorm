#![allow(unused_imports, dead_code, unused_variables)]
use anyhow::Result;
use geekorm::{prelude::*, queries::QueryOrder};
use geekorm_derive::GeekTable;

#[derive(Debug, GeekTable)]
pub struct User {
    pub name: String,
    pub age: i32,
}

fn main() -> Result<()> {
    println!("Starting...\n");

    let table = User::table();
    println!("Table :: {:?}\n", table);

    let query = User::create().build()?;
    println!("Create :: `{}`", query);

    let select = User::select()
        // order by name asc
        .order_by("age", QueryOrder::Asc)
        // build
        .build()?;

    println!("Select :: `{}`", select);

    // let user = User::select()
    //     .where(User::Column::name == "John" || (User::Column::age > 20 && User::Column::age < 30))
    //     .execute()?;

    Ok(())
}
