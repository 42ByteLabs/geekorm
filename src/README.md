 GeekORM is a simple and lightweight ORM for Rust. It is designed to be simple and easy to use.

 This project is still in the early stages of development, and as such, it may not be suitable for all use cases.

### Features

- Simple and lightweight
- Uses derive macros for easy table creation
- Helper Functions to make things easy
  - Automatic `new` function
    - feature: `new`
  - Automatic `select_by_{field}` functions
    - feature: `helpers`
- Multi-Backends Support
  - [LibSQL](https://github.com/tursodatabase/libsql)
- Query builder pattern for building SQL queries
  - Only supports SQLite at the moment

### Example

 Here is a simple example of how to use GeekORM:

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
     .where_gt("age", 20)
     .order_by("age", QueryOrder::Asc)
     .limit(10)
     .build()
     .expect("Failed to build query");
 println!("Select Users Query: {}", select_user);

 // Print the values that will be used in the query
 // This is useful for passing values to a database driver or connection in the correct order
 println!("Select Users Values: {:?}", select_user.values);
 ```

### Unsupported Features

 If you are building a complex application, GeekORM may not be the best choice for you.
 GeekORM is designed to be simple and lightweight, and as such, it does not support some of the more advanced features that other ORMs may offer.

 Here is a list of some of the features that GeekORM does not support (but may support in the future):

- Automatic Migrations
- Relationships (e.g. One-to-Many, Many-to-Many)
- Transactions
- Connection Pooling

 If there are any features you would like to see in GeekORM, please open an issue on the GitHub repository.
