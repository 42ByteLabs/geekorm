#![allow(unused_imports, dead_code, unused_variables)]
extern crate geekorm;

use anyhow::Result;
// Imports
use geekorm::Tables;
use geekorm::{ForeignKey, GeekTable, PrimaryKey, QueryOrder, Table, TableBuilder};

#[derive(Debug, Default, GeekTable)]
pub struct User {
    pub id: PrimaryKey,
    pub name: String,
    pub age: i32,
    pub postcode: Option<String>,
    pub profile_id: ForeignKey<Profile>,
}

#[derive(Debug, Default, GeekTable)]
pub struct Profile {
    pub random_name: PrimaryKey,
    pub bio: String,
}

fn main() -> Result<()> {
    println!("Starting...\n");
    let tables = Tables::init(vec![User, Profile]);

    println!("Tables :: {}", tables.tables.len());
    for table in tables.tables {
        println!("Table :: {:#?}", table.get_primary_key());
    }
    println!("");

    let table_user = User::table();
    // println!("Table :: {:#?}\n", table_user);

    let table_profile = Profile::table();
    // println!("Table :: {:#?}\n", table_profile);

    let query = User::create().build()?;
    println!("Create :: `{}`\n", query);

    let select = User::select()
        .where_ne("name", "Mathew")
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
