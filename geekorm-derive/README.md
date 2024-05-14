# GeekORM Derive

The `geekorm_derive` crate is for all pre-processing derive macros used by `geekorm` at build time.

## Default Features

### Generate Query Methods

By default, the following methods are generated for the struct:

- `create()`: Create Query
- `select()`: Select Query
- `all()`: Select all rows in a table
- `insert()`: Insert Query 
- `update()`: Update Query
- `count()`: Count the number of rows

These are all defined by the `geekorm_core::QueryBuilderTrait` trait.

```rust
use geekorm::{GeekTable, PrimaryKeyInteger};
use geekorm::prelude::*;

#[derive(GeekTable, Default)]
struct Users {
    id: PrimaryKeyInteger,
    name: String,
    age: i32,
    occupation: String,
}

// Create a new table query
let create = Users::create().build()
    .expect("Failed to build CREATE TABLE query");

// Select data from the table
let select = Users::select()
    .where_eq("name", "geekmasher")
    .build()
    .expect("Failed to build SELECT query");

// Create a default User
let mut user = Users::default();

// Insert data 
let insert = Users::insert(&user);

// Update query
user.name = String::from("42ByteLabs");
let update = Users::update(&user);
```

## Feature - Automatic New Struct Function

When the `new` feature is enabled, the following methods are generated for the struct:

- `PrimaryKey<T>` fields are not generated
- `Option<T>` fields are not generated

```rust
use geekorm::{GeekTable, PrimaryKeyInteger};
use geekorm::prelude::*;

#[derive(GeekTable)]
struct Users {
    id: PrimaryKeyInteger,
    name: String,
    age: i32,
    occupation: String,
    country: Option<String>,
}

let user = Users::new(
    String::from("geekmasher"),
    42,
    String::from("Software Developer")
);
```

# Feature - Generated Helper Methods

When the `helpers` feature is enabled, the following helper methods are generated for the struct:

_Note:_ This is a very experimental feature and might change in the future.

```rust
use geekorm::{GeekTable, PrimaryKeyInteger};
use geekorm::prelude::*;

#[derive(GeekTable)]
struct Users {
    id: PrimaryKeyInteger,
    name: String,
    age: i32,
    occupation: String,
}

// Select by column helper function
let user = Users::select_by_name("geekmasher");
# assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE name = ?;"));
let user = Users::select_by_age(42);
# assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE age = ?;"));
let user = Users::select_by_occupation("Software Developer");
# assert_eq!(user.query, String::from("SELECT id, name, age, occupation FROM Users WHERE occupation = ?;"));
```

## Feature - Generate Random Data for Column

When using the `rand` feature, you can automatically generate random strings and use 

```rust
use geekorm::prelude::*;
use geekorm::{GeekTable, PrimaryKeyInteger};

#[derive(GeekTable, Debug)]
pub struct Users {
    id: PrimaryKeyInteger,
    name: String,
    #[geekorm(rand, rand_length = 42, rand_prefix = "token")]
    token: String
}

let user = Users::new(String::from("geekmasher"));
println!("{}", user.token);
# assert_eq!(user.token.len(), 48);
```

**`rand` attributes:**

- `rand`: Sets the String field as a randomly generated value
- `rand_length`: Sets the length of the randomly generated string
    - Default: `32`
- `rand_prefix`: Sets a prefix to the randomly generated string  
    - Default: None

## Feature - Generate Hashs for storing passwords

When using the `hash` feature, you can automatically hash passwords to make sure they are stored securely.

```rust
use geekorm::prelude::*;
use geekorm::{GeekTable, PrimaryKeyInteger};

#[derive(GeekTable, Debug)]
pub struct Users {
    id: PrimaryKeyInteger,
    username: String,

    #[geekorm(hash, hash_algorithm = "Pbkdf2")]
    password: String,
}

# fn main() -> Result<(), geekorm::Error> {
let mut user = Users::new(String::from("geekmasher"), String::from("password"));
# assert_eq!(user.password.len(), 95);

// Update password
user.hash_password("newpassword");

// Verify password
if user.check_password("newpassword")? {
   println!("Password is correct");
} else {
   println!("Password is incorrect");
}

# Ok(())
# }
```

**`hash` attributes:**

- `hash` or `password`: Sets the String field as a hashable value
- `hash_algorithm`: Set the algorithm to use
    - Default: `Pbkdf2`

