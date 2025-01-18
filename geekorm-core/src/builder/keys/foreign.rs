use core::fmt;
use std::fmt::{Debug, Display};

use serde::{de::Visitor, Deserialize, Serialize, Serializer};

use crate::{PrimaryKey, TableBuilder};

/// Foreign Key Type
///
/// ```rust
/// use geekorm::prelude::*;
///
/// #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
/// }
///
/// #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
/// struct Posts {
///     id: PrimaryKeyInteger,
///     title: String,
///     /// Foreign Key to the Users table
///     /// i32 as the key type, and Users as the data type
///     #[geekorm(foreign_key = "Users.id")]
///     user: ForeignKey<i32, Users>,
/// }
///
/// // Create the Posts table with the foreign key referencing the Users table (Users.id)
/// let create_posts_query = Posts::query_create().build()
///     .expect("Failed to build query");
/// # assert_eq!(
/// #     create_posts_query.to_str(),
/// #     "CREATE TABLE IF NOT EXISTS Posts (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, user INTEGER NOT NULL, FOREIGN KEY (user) REFERENCES Users(id));"
/// # );
///
/// // Use the foreign key to and join the tables together
/// // to get the user posts
/// let user_posts = Users::query_select()
///     .order_by("name", geekorm::QueryOrder::Asc)
///     .build()
///     .expect("Failed to build query");
///
/// println!("User posts query: {:?}", user_posts);
/// // ...
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ForeignKey<T, D>
where
    T: serde::Serialize + 'static,
    D: TableBuilder,
{
    /// Foreign Key Value
    pub key: T,
    /// Foreign Key Data Type
    pub data: D,
}

/// Foreign Key as an Integer
pub type ForeignKeyInteger<T> = ForeignKey<u64, T>;

/// Old Foreign Key as an Integer
pub(crate) type ForeignKeyIntegerOld<T> = ForeignKey<i32, T>;

/// Foreign Key as a String
pub type ForeignKeyString<T> = ForeignKey<String, T>;

/// Foreign Key as an Uuid
#[cfg(feature = "uuid")]
pub type ForeignKeyUuid<T> = ForeignKey<uuid::Uuid, T>;

impl<T, D> Debug for ForeignKey<T, D>
where
    T: serde::Serialize + Debug + 'static,
    D: TableBuilder,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForeignKey({:?})", self.key)
    }
}

impl<T, D> Display for ForeignKey<T, D>
where
    T: serde::Serialize + Display + 'static,
    D: TableBuilder,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.data.get_table().name, self.key)
    }
}

impl<D> Default for ForeignKey<u64, D>
where
    D: TableBuilder + Default,
{
    fn default() -> Self {
        Self {
            key: Default::default(),
            data: Default::default(),
        }
    }
}

impl<D> Default for ForeignKey<String, D>
where
    D: TableBuilder + Default,
{
    fn default() -> Self {
        Self {
            key: Default::default(),
            data: Default::default(),
        }
    }
}

impl<D> Default for ForeignKey<i32, D>
where
    D: TableBuilder + Default,
{
    fn default() -> Self {
        Self {
            key: Default::default(),
            data: Default::default(),
        }
    }
}

impl<D> ForeignKey<u64, D>
where
    D: TableBuilder + Default,
{
    /// Create a new foreign key with an integer
    pub fn new(value: u64) -> Self {
        Self {
            key: value,
            data: Default::default(),
        }
    }
}

impl<D> ForeignKey<i32, D>
where
    D: TableBuilder + Default,
{
    /// Create a new foreign key with an integer
    pub fn new(value: i32) -> Self {
        Self {
            key: value,
            data: Default::default(),
        }
    }
}

impl<D> ForeignKey<String, D>
where
    D: TableBuilder + Default,
{
    /// Create a new foreign key with a String
    pub fn new(value: String) -> Self {
        Self {
            key: value,
            data: Default::default(),
        }
    }
}

impl<D> From<u64> for ForeignKey<u64, D>
where
    D: TableBuilder + Default,
{
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<D> From<i32> for ForeignKey<i32, D>
where
    D: TableBuilder + Default,
{
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl<D> From<String> for ForeignKey<String, D>
where
    D: TableBuilder + Default,
{
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<D> From<&str> for ForeignKey<String, D>
where
    D: TableBuilder + Default,
{
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl<D> From<ForeignKey<u64, D>> for u64
where
    D: TableBuilder,
{
    fn from(value: ForeignKey<u64, D>) -> Self {
        value.key
    }
}

impl<D> From<PrimaryKey<u64>> for ForeignKey<u64, D>
where
    D: TableBuilder + Default,
{
    fn from(value: PrimaryKey<u64>) -> Self {
        Self::new(value.value)
    }
}

impl<D> From<ForeignKey<i32, D>> for i32
where
    D: TableBuilder,
{
    fn from(value: ForeignKey<i32, D>) -> Self {
        value.key
    }
}

impl<D> From<&ForeignKeyIntegerOld<D>> for i32
where
    D: TableBuilder,
{
    fn from(value: &ForeignKeyIntegerOld<D>) -> Self {
        value.key
    }
}

impl<D> From<PrimaryKey<i32>> for ForeignKey<i32, D>
where
    D: TableBuilder + Default,
{
    fn from(value: PrimaryKey<i32>) -> Self {
        Self::new(value.value)
    }
}

impl<D> From<PrimaryKey<String>> for ForeignKey<String, D>
where
    D: TableBuilder + Default,
{
    fn from(value: PrimaryKey<String>) -> Self {
        Self::new(value.value)
    }
}

impl<D> Serialize for ForeignKeyInteger<D>
where
    D: TableBuilder + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.key)
    }
}
impl<D> Serialize for ForeignKeyIntegerOld<D>
where
    D: TableBuilder + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.key)
    }
}

impl<'de, T> Deserialize<'de> for ForeignKeyInteger<T>
where
    T: TableBuilder + Default + Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<ForeignKeyInteger<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ForeignKeyVisitor;

        impl<'de> Visitor<'de> for ForeignKeyVisitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer representing a foreign key")
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value as u64)
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as u64)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as u64)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as u64)
            }
        }

        Ok(ForeignKey::from(
            deserializer.deserialize_u64(ForeignKeyVisitor)?,
        ))
    }
}

impl<'de, T> Deserialize<'de> for ForeignKeyIntegerOld<T>
where
    T: TableBuilder + Default + Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<ForeignKeyIntegerOld<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ForeignKeyVisitor;

        impl<'de> Visitor<'de> for ForeignKeyVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer representing a foreign key")
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(value)
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as i32)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as i32)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v as i32)
            }
        }

        Ok(ForeignKeyIntegerOld::from(
            deserializer.deserialize_i32(ForeignKeyVisitor)?,
        ))
    }
}
