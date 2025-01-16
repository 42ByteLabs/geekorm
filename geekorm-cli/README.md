<!-- markdownlint-disable -->
<div align="center">
<h1>GeekORM CLI</h1>

<img src="../assets/geekorm.png" width="450" title="GeekORM Logo">

[![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)][github]
[![Crates.io Version](https://img.shields.io/crates/v/geekorm-cli?style=for-the-badge)][crates-io]
[![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/geekorm-cli?style=for-the-badge)][crates-io]
[![GitHub Stars](https://img.shields.io/github/stars/42ByteLabs/geekorm?style=for-the-badge)][github]
[![GitHub Issues](https://img.shields.io/github/issues/42ByteLabs/geekorm?style=for-the-badge)][github-issues]
[![Licence](https://img.shields.io/github/license/Ileriayo/markdown-badges?style=for-the-badge)][license]

</div>
<!-- markdownlint-restore -->

## Overview

[GeekORM CLI][crates-io] is a simple Command Line Interface for managing GeekORM migrations.

## 📦 Installation

You can install the CLI from [crates.io][crates]:

```bash
cargo install geekorm-cli
```

## Choosing a Mode

The `geekorm-cli` can generate a `crate` or a `module` for you to use.

There are a few differences between the two modes:

- `module` mode is the default and is the easiest to use.
- `crate` mode allows migrations to be used in many projects.
- `module` mode can use the models from your project.
  - This can help with data migrations and other tasks.

## Initializing Your Project

To get started, you'll need to initialize your project with the `geekorm-cli` command.

```bash
geekorm-cli init
```

This will prompt you to enter some information about your setup and will generate a `crate` or a `module` for you to use.

Once information will be stored in a `.geekorm.yml` file in the root of your project.

**Example `.geekorm.yml` file**

```yaml
# GeekORM CLI Configuration

# Mode can be `crate` or `module`
mode: module
# Name of the crate or module (allows for custom naming)
name: db
# Backend Database Type
database: sqlite
# Driver(s) to use
drivers:
- libsql
```

> [!NOTE]
> Migrations uses your projects version to create migrations.

## Setup - Module Mode

If you choose to use the `module` mode, you'll see a new directory in your `./src` project called `db` (or whatever you named it).

Inside this directory, you'll see a new file called `mod.rs` that contains the following code:

```rust
#![doc = r" GeekORM Database Migrations"]
#![allow(unused_imports, unused_variables)]
use geekorm::prelude::*;
pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
where
    T: geekorm::GeekConnection<Connection = T> + 'a,
{
    Ok(())
}
```

> [!NOTE]
> You will need to add `db` to your `lib.rs` or `main.rs` file to use it.
> This might mean you need to add a `mod db;` line to the top of your file.

This mode will also add/update your dependencies in your `Cargo.toml` file.

```toml
[dependencies]
geekorm = { version = "0.9.0", features = ["all", "backends", "migrations"] }
lazy_static = "1"
```

## Setup - Crate Mode

If you choose to use the `crate` mode, you'll see a new directory in your project called `db` (or whatever you named it).
This directory should be added to your `Cargo.toml` as a dependency.

```toml
db = { version = "0.1.0", path = "db" }
```

The `db` directory will contain a `Cargo.toml` file that will install a few dependencies for you.

```toml
[package]
name = "db"
version = "0.1.0"
edition = "2023"

[dependencies]
# If you selected a backend driver, it will be added here too
geekorm = { version = "0.9.0", features = ["all", "backends", "migrations"] }
# This is required for `geekorm` migrations to work
lazy_static = "1"

# Backend Drivers will also be added here
# libsql = "0.6.0"
# rusqlite = "0.32.0"
```

Inside this directory, you'll see new Cargo project along with a `lib.rs` file that contains the following code:

```rust
#![doc = r" GeekORM Database Migrations"]
#![allow(unused_imports, unused_variables)]
use geekorm::prelude::*;
pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
where
    T: geekorm::GeekConnection<Connection = T> + 'a,
{
    Ok(())
}
```

It currently has no migrations, but they will be added when you use the [migrate command][#migrations].

### 🦀 Rust and 🧪 SQL Structure

GeekORM will generate and manage various files in your project to help you manage your models and migrations.

```text
│   # `db` is the default name, but you can change it
├── db/
│   │   # `lib.rs` for `crate` mode
│   ├── lib.rs
│   │   # `mod.rs` for `module` mode
│   ├── mod.rs
│   │   # migration directory for a specific version
│   └── v{version}/
│       │   # migration module for this version
│       └── mod.rs
│       │   # SQL batch query to create the database (from scratch)
│       └── create.sql  
│       │   # SQL batch query to upgrade the database to this version (previous version)
│       └── upgrade.sql
```

Both the Rust and SQL files will be generated for you, but you can customize them if you want.

## Migrations

After you have initialized your project, you can start using the `geekorm-cli` to manage your migrations.
This will work in both `crate` and `module` mode.

```bash
geekorm-cli migrate
```

This will indentify the latest version of your database and will intactively prompt you what do to to create a new migration.

**Example: New Column Migration**

```text
[ERROR] Database is out of date!
[INFO ] Errors found, creating a schema migration...
[INFO ] Error: Missing Column `Users.last_login`
[INFO ] Prompting for missing column: `MissingColumn { table: "Users", column: "last_login" }`
? Alter Column: ›  
❯ Create
  Rename
  Skip
```

You can choose different types of migrations which once completed will be added to the `db` directory.

Here is an example of a new column migration that was created in the `module` mode.

**Path: `src/db/v0_1_1/upgrade.sql`**

```sql
-- This migration will update the schema

ALTER TABLE Users ADD COLUMN last_login TEXT NOT NULL DEFAULT '';
```

### 📚 Adding Data Migrations

**WIP - Not yet implemented**

## Test Migrations

One of the best features of the `geekorm-cli` is the ability to test your migrations.

```bash
geekorm-cli test
```

This will create a new database in memory and will run all of your migrations from the beginning to the latest version.

If there are any errors, it will stop and report the error to you.

## Updating Codegeneration

If `geekorm` is updated, you might want to run the update commend to update the codegeneration in your project.

```bash
geekorm-cli update
```

This will update any codegeneration that was done by the `geekorm-cli` in your project.

## Using in your project

Once you have setup and added your migrations, you can start using the `geekorm` library in your project.

### 🏎️ Setting Up Your Database Connection

```rust
use anyhow::Result;
use geekorm::prelude::*;

// Import the `db` module if you are using `module` mode
mod db;

// ... 

#[tokio::main]
async fn main() -> Result<()> {
    // Setup the database and connection
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open database");

    // Initialize or migrate the database using the `crate` or `module`.
    // This is done using the `geekorm-cli` function
    db::init(&conn).await?;

    // All done!

    Ok(())
}
```


<!-- Resources -->

[license]: ../LICENSE
[crates-io]: https://crates.io/crates/geekorm-cli
[rust-lang]: https://www.rust-lang.org/
[github]: https://github.com/42ByteLabs/geekorm
[github-issues]: https://github.com/42ByteLabs/geekorm/issues
