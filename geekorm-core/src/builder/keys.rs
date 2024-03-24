/// Primary Key
pub mod primary;
pub use primary::PrimaryKey;

use crate::TableBuilder;

/// Foreign Key
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
///     user_id: ForeignKey<User>,
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
#[derive(Debug, Clone, Default)]
pub struct ForeignKey<T>
where
    T: TableBuilder,
{
    table_type: T,
    table_name: String,
}

impl<T> ForeignKey<T>
where
    T: TableBuilder,
{
    /// Get the foreign key column name (table + primary key)
    pub fn get_column(&self) -> String {
        format!(
            "{}_{}",
            self.table_name,
            self.table_type
                .get_primary_key()
                .unwrap_or_else(|| "id".to_string())
        )
    }
}
