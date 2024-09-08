<!-- markdownlint-disable -->
<div align="center">
<h1>GeekORM</h1>

<img src="./assets/geekorm.png" width="450" title="GeekORM Logo">

[![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)][github]
[![Crates.io Version](https://img.shields.io/crates/v/geekorm?style=for-the-badge)][crates-io]
[![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/geekorm?style=for-the-badge)][crates-io]
[![GitHub Stars](https://img.shields.io/github/stars/42ByteLabs/geekorm?style=for-the-badge)][github]
[![GitHub Issues](https://img.shields.io/github/issues/42ByteLabs/geekorm?style=for-the-badge)][github-issues]
[![Licence](https://img.shields.io/github/license/Ileriayo/markdown-badges?style=for-the-badge)][license]

</div>
<!-- markdownlint-restore -->

## Overview

[GeekORM][crates-io] is a simple [Object Relation Mapper][docs-orm] for empowering your [Rust][rust-lang] development.

## ✨ Features

- Focus on simplicity
- Rely on Derive Macros to generate code for your structs
  - Using `Table`
  - Using `Data`
- Dynamically build queries
  - `Select`, `Create`, `Update`, and `Insert` queries
- [Extensive crate features](#-create-features)
- Field Attribute Helpers
  - `foreign_key`: Set the foreign key for a join
  - [`rand`][docs-rand]: Generate random strings (set lenght, set prefix, set enviroment)
  - [`hash` or `password`][docs-hash]: Generate secure Hashes of passwords (set algorithm)
- Support for Backends
  - [`rusqlite`][lib-rusqlite]
  - [`libsql`][lib-libsql] ([Turso][web-turso])
- [Documentation][docs]

## 📦 Usage

You can install the library from [crates.io][crates]:

```bash
cargo add geekorm
```

### Manual - GitHub

```bash
cargo install --git https://github.com/42ByteLabs/geekorm
```

## 🏃 Getting Started

Once you have installed `geekorm`, you can start using the derive macros like the following:

```rust
use anyhow::Result;
use geekorm::prelude::*;

#[derive(Table, Debug, Default, serde::Serialize, serde::Deserialize)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    username: String,

    #[geekorm(hash)]
    password: String,

    #[geekorm(new = "UserType::User")]
    user_type: UserType,

    #[geekorm(new = "chrono::Utc::now()")]
    created_at: chrono::DateTime<chrono::Utc>,

    postcode: Option<String>,
}

#[derive(Data, Debug, Default, Clone)]
enum UserType {
    Admin,
    #[default]
    User,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the database and connection
    let db = libsql::Builder::new_local(":memory:").build().await
        .expect("Failed to create database");
    let connection = db.connect()
        .expect("Failed to connect to database");

    // Create the table in the database
    Users::create_table(&connection).await?;

    // Creating a new User
    let mut user = Users::new("GeekMasher", "ThisIsNotMyPassword");
    // Saving the new User in the database
    user.save(&connection).await?;
    // Print the Primary Key value set by the database (auto_increment)
    println!("User ID: {:?}", user.id);

    // Updating the User
    user.user_type = UserType::Admin;
    user.update(&connection).await?;

    // Fetch the Admin Users
    let admin_users = Users::fetch_by_user_type(&connection, UserType::Admin).await?;
    println!("Admin Users: {:?}", admin_users);

    // Helper functions built right into the struct by GeekORM
    user.hash_password("ThisIsStillNotMyPassword")?;

    // Go back to basics and build your own queries dynamically using
    // the QueryBuilder built into GeekORM
    let query = Users::query_select()
        .where_eq("username", "GeekMasher")
        .order_by("id", geekorm::QueryOrder::Desc)
        .limit(1)
        .build()?;

    // Execute the query and return the results
    let users = Users::query(&connection, query).await?;
    println!("Users: {:?}", users);

    Ok(())
}
```

### 🏄 Create Features

There are a number of opt-in features supported by GeekORM.
Features can be added either [using `cargo add geekorm -F all`][docs-cargo-add] or added them directly in your `Cargo.toml` file.

- `all`: Enable all the major stable features
- [`new`][docs-new]: Generate `Table::new(...)` functions
- [`helpers`][docs-helpers]: Generate a number of helper functions
  - Select `Table::select_by_primary_key()`
  - Select column `Table::select_by_{field}()`
- [`rand`][docs-rand]: Support Generating random strings
- [`hash`][docs-hash]: Support Generating password hashes
- Backends
  - `libsql`: Add LibSQL backend support

## 🧑‍🤝‍🧑 Maintainers / Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://geekmasher.dev"><img src="https://avatars.githubusercontent.com/u/2772944?v=4?s=100" width="100px;" alt="Mathew Payne"/><br /><sub><b>Mathew Payne</b></sub></a><br /><a href="#code-GeekMasher" title="Code">💻</a> <a href="#review-GeekMasher" title="Reviewed Pull Requests">👀</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/MsGeekMasher"><img src="https://avatars.githubusercontent.com/u/93775622?v=4?s=100" width="100px;" alt="Cale"/><br /><sub><b>Cale</b></sub></a><br /><a href="#design-MsGeekMasher" title="Design">🎨</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## 🦸 Support

Please create [GitHub Issues][github-issues] if there are bugs or feature requests.

This project uses [Semantic Versioning (v2)][semver] and with major releases, breaking changes will occur.

## 📓 License

This project is licensed under the terms of the MIT open source license.
Please refer to [MIT][license] for the full terms.

<!-- Resources -->

[license]: ./LICENSE
[crates-io]: https://crates.io/crates/geekorm
[docs]: https://docs.rs/geekorm/latest/geekorm
[rust-lang]: https://www.rust-lang.org/
[semver]: https://semver.org/
[github]: https://github.com/42ByteLabs/geekorm
[github-issues]: https://github.com/42ByteLabs/geekorm/issues
[crates]: https://crates.io
[docs-orm]: https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping
[docs-cargo-add]: https://doc.rust-lang.org/cargo/commands/cargo-add.html#dependency-options

[docs-new]: https://docs.rs/geekorm-derive/latest/geekorm_derive/derive.GeekTable.html#generate-new-rows
[docs-helpers]: https://docs.rs/geekorm-derive/latest/geekorm_derive/derive.GeekTable.html#generated-helper-methods
[docs-hash]: https://docs.rs/geekorm-derive/latest/geekorm_derive/derive.GeekTable.html#generate-hash-for-storing-passwords
[docs-rand]: https://docs.rs/geekorm-derive/latest/geekorm_derive/derive.GeekTable.html#generate-random-data-for-column

[lib-libsql]: https://github.com/tursodatabase/libsql
[lib-rusqlite]: https://github.com/rusqlite/rusqlite
[web-turso]: https://turso.tech/
