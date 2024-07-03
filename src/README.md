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
use geekorm::prelude::*;

#[derive(Table, Debug, Default)]
struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKey<i32>,

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

#[derive(Data, Debug, Default)]
enum UserType {
    Admin,
    #[default]
    User,
}

#[tokio::main]
async fn main() -> Result<(), geekorm::Error> {
    // Setup the database and connection
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let connection = db.connect()?;
   
    // Creating a new User
    let mut user = Users::new("GeekMasher", "ThisIsNotMyPassword");
    // Saving the new User in the database
    user.save(&connection).await?;
    // Print the Primary Key value set by the database (auto_increment)
    println!("User ID: {}", user.id);

    // Updating the User
    user.user_type = UserType::Admin;
    user.update(&connection).await?;

    // Fetch the Admin Users
    let admin_users = Users::fetch_by_user_type(&connection, UserType::Admin).await?;

    // Helper functions built right into the struct by GeekORM
    user.hash_password("ThisIsStillNotMyPassword");

    // Go back to basics and build your own queries dynamically using 
    // the QueryBuilder built into GeekORM
    let query = Users::query_select()
        .where_eq("username", "GeekMasher")
        .order_by("id", Order::Desc)
        .limit(1)
        .build()?;
    // Execute the query and return the results
    let users = Users::query(&connection, query).await?;

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
