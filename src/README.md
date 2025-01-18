# GeekORM

GeekORM is a simple and lightweight ORM for Rust. It is designed to be simple and easy to use.

This project is still in the early stages of development, and as such, it may not be suitable for all use cases.

## ✨ Features

- Focus on simplicity
- Rely on Derive Macros to generate code for your structs
  - Using `Table`
  - Using `Data`
- Dynamically generate functions and corresponding SQL queries
  - `.save(...)` - Inserting new rows
  - `.update(...)` - Updating existing rows
  - `.delete(...)` - Deleting rows
- Support for Backends Drivers
  - `rusqlite`
  - `libsql`
- Automatic Migration Generation
  - `geekorm-cli init` - Setup your migrations
- Extensive crate features
  - `rand`: Generate random strings (set lenght, set prefix, set enviroment)
  - `hash` or `password`: Generate secure Hashes of passwords (set algorithm)
  - and more...

## 📦 Installation

### 🦀 Library

You can install the library from [crates.io][crates]:

```bash
cargo add geekorm
```

### 🛠️ CLI

If you want to manage your models and migrations using `geekorm`, you'll need to install the `geekorm-cli` command line tool.

```bash
cargo install geekorm-cli
```

## 🏃 Getting Started

GeekORM is easy to setup and use in your Rust project.

### 🏎️ Setting Up Migrations

The first thing you'll need to decide is if you want to use the `geekorm-cli` to manage your migrations or if you want to manage them manually.

You can use the `geekorm-cli` to help you manage your migrations.

```bash
geekorm-cli init
```

This will prompt you to enter some information about your setup and will generate a `crate` or a `module` for you to use.
Once you have setup your project, 2 new commands will be available to you:

```bash
# Generate a new migration (creates a new folders in your migrations directory)
geekorm-cli migrate 
# Validate your migrations (runs from your initial migration to the latest)
geekorm-cli test
```

### 🚀 Writing your first model

Once you have installed `geekorm`, you can start using the derive macros like the following:

```rust
# #[cfg(feature = "libsql")] {
use anyhow::Result;
use geekorm::prelude::*;

/// Using the `Table` derive macro to generate the `Users` table
#[derive(Table, Debug, Default, serde::Serialize, serde::Deserialize)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,
    /// Unique username field
    #[geekorm(unique)]
    username: String,
    /// Password field with automatic hashing
    #[geekorm(hash)]
    password: String,
    /// User Type Enum (defaults to `User`)
    #[geekorm(new = "UserType::User")]
    user_type: UserType,
}

#[derive(Data, Debug, Default, Clone)]
enum UserType {
    Admin,
    Moderator,
    #[default]
    User,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the database and connection
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open database");

    // Initialize or migrate the database using the `crate` or `module`.
    // This is done using the `geekorm-cli` function
    db::init(&conn).await?;
    // [OR] You can create the tables manually
    Users::create_table(&conn).await?;

    // Use the generated `new` function to create a new User
    // using the default values set in the struct.
    let mut user = Users::new("GeekMasher", "ThisIsNotMyPassword");
    // Saving the new User in the database
    user.save(&connection).await?;
    // Print the Primary Key value set by the database (auto_increment)
    println!("User ID: {:?}", user.id);

    // Updating the Users account type to Admin
    user.user_type = UserType::Admin;
    user.update(&connection).await?;

    // Fetch the Admin Users
    let admin_users = Users::fetch_by_user_type(&connection, UserType::Admin).await?;
    println!("Admin Users: {:?}", admin_users);

    // Counts the number of Users in the database
    let total_users = Users::total(&connection).await?;
    println!("Total Users: {:?}", total_users);

    Ok(())
}
# }
```

### Unsupported Features

 If you are building a complex application, GeekORM may not be the best choice for you.
 GeekORM is designed to be simple and lightweight, and as such, it does not support some of the more advanced features that other ORMs may offer.

 Here is a list of some of the features that GeekORM does not support (but may support in the future):

- Transactions
- Connection Pooling

 If there are any features you would like to see in GeekORM, please open an issue on the GitHub repository.
