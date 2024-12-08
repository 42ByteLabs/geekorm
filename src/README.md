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
#[cfg(feature = "libsql")] 
{
use anyhow::Result;
use geekorm::prelude::*;

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
    /// Optional postcode field (nullable in the database)
    postcode: Option<String>,
    /// Randomly generated session token
    #[geekorm(rand = "42", rand_prefix = "session_")]
    session: String,

    /// Created and Updated timestamps
    #[geekorm(new = "chrono::Utc::now()")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[geekorm(new = "chrono::Utc::now()", on_update = "chrono::Utc::now()")]
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Data, Debug, Default, Clone)]
enum UserType {
    Admin,
    Modirator,
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

    // Use the generated `new` function to create a new User
    // using the default values set in the struct.
    let mut user = Users::new("GeekMasher", "ThisIsNotMyPassword");
    // Saving the new User in the database
    user.save(&connection).await?;
    // Print the Primary Key value set by the database (auto_increment)
    println!("User ID: {:?}", user.id);

    // Updating the Users postcode (optional field)
    user.postcode = Some("SW1A 1AA".to_string());
    user.update(&connection).await?;

    // Fetch the Admin Users
    let admin_users = Users::fetch_by_user_type(&connection, UserType::Admin).await?;
    println!("Admin Users: {:?}", admin_users);

    // Counts the number of Users in the database
    let total_users = Users::total(&connection).await?;

    // Enums are used to help columns with a limited set of values
    // and GeekORM will handle the conversion for you.
    user.user_type = UserType::Admin;
    // or you can use the `.from()` or `.into()` functions
    user.user_type = UserType::from("Admin");
    user.user_type = "Admin".into();

    // GeekORM offers a number of helper functions to make your life easier.
    
    // Search unique fields or search tagged fields
    let search = Users::search(&connection, "GeekMasher").await?;

    // Automatically hashing passwords for you.
    user.hash_password("ThisIsStillNotMyPassword")?;

    // Automatically generate random strings for you.
    user.regenerate_session();

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
# }
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
