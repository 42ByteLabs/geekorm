use crate::{TableBuilder, TablePrimaryKey};

/// Foreign Key Type
///
/// ```rust
/// use geekorm::{GeekTable, ForeignKey, PrimaryKey};
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct User {
///     id: PrimaryKey,
///     name: String,
/// }
///
/// #[derive(GeekTable)]
/// struct Post {
///     id: PrimaryKey,
///     title: String,
///     user_id: ForeignKey<&i32>,
/// }
///
/// // Use the foreign key to and join the tables together
/// // to get the user posts
/// let user_posts = User::select()
///     .columns(vec!["User.name", "Post.title"])
///     .join(Post::table())
///     .order_by("name", geekorm::QueryOrder::Asc)
///     .build()
///     .expect("Failed to build query");
///
/// println!("User posts query: {:?}", user_posts);
///
/// ```
#[derive(Clone)]
pub struct ForeignKey<T>
where
    T: TableBuilder + TablePrimaryKey + 'static,
{
    table_type: &'static T,
    table_name: String,
}

impl<T> ForeignKey<T>
where
    T: TableBuilder + TablePrimaryKey + 'static,
{
    /// Create a new foreign key with another table
    pub fn new(table: &'static T) -> Self {
        Self {
            table_type: table,
            table_name: T::table_name(),
        }
    }

    /// Get the foreign key column name (table + primary key)
    pub fn get_column(&self) -> String {
        format!(
            "{}_{}",
            self.table_name,
            self.table_type.get_table().get_primary_key()
        )
    }
}
