<!-- markdownlint-disable -->
<div align="center">
<h1>GeekORM</h1>

[![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)][github]
[![Crates.io Version](https://img.shields.io/crates/v/geekorm?style=for-the-badge)][crates-io]
[![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/geekorm?style=for-the-badge)][crates-io]
[![GitHub Stars](https://img.shields.io/github/stars/42ByteLabs/geekorm?style=for-the-badge)][github]
[![GitHub Issues](https://img.shields.io/github/issues/42ByteLabs/geekorm?style=for-the-badge)][github-issues]
[![Licence](https://img.shields.io/github/license/Ileriayo/markdown-badges?style=for-the-badge)][license]

<img src="./assets/geekorm.png" width="450" title="GeekORM Logo">

</div>
<!-- markdownlint-restore -->

## Overview

[GeekORM][crates-io] is a simple [Object Relation Mapper][docs-orm] for empowering your [Rust][rust-lang] development.

## ✨ Features

- Focus on simplicity
- Rely on Derive Macros to generate code for your structs
  - Using `GeekTable`
- Dynamically build queries
  - `Select`, `Create`, `Update`, and `Insert` queries
- [Extensive crate features](#-create-features)
- Field Attribute Helpers
  - `foreign_key`: Set the foreign key for a join
  - [`rand`][docs-rand]: Generate random strings (set lenght, set prefix, set enviroment)
  - [`hash` or `password`][docs-hash]: Generate secure Hashes of passwords (set algorithm)
- Support for Backends
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
use geekorm::prelude::*;
use geekorm::{QueryOrder, PrimaryKeyInteger};

#[derive(Debug, Clone, GeekTable)]
struct Users {
   id: PrimaryKeyInteger,
   username: String,
   email: String,
   age: i32,
   postcode: Option<String>,
}

// Use the `create` method to build a CREATE TABLE query
let create_table = Users::query_create().build()
    .expect("Failed to build create table query");
println!("Create Table Query: {}", create_table);

// Use the `select` method to build a SELECT query along with different conditions
// and ordering
let select_user = Users::query_select()
    .where_eq("username", "geekmasher")
    .and()
    .where_gt("age", 42)
    .order_by("age", QueryOrder::Asc)
    .limit(10)
    .build()
    .expect("Failed to build query");

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
[web-turso]: https://turso.tech/
