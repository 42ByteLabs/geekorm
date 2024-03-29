use core::fmt;
use std::fmt::{Debug, Display};

use serde::{de::Visitor, Deserialize, Serialize, Serializer};

use crate::PrimaryKey;

/// Foreign Key Type
///
/// ```rust
/// use geekorm::{GeekTable, ForeignKey, PrimaryKeyInteger};
/// use geekorm::prelude::*;
///
/// #[derive(GeekTable)]
/// struct Users {
///     id: PrimaryKeyInteger,
///     name: String,
/// }
///
/// #[derive(GeekTable)]
/// struct Posts {
///     id: PrimaryKeyInteger,
///     title: String,
///     #[geekorm(foreign_key = "Users.id")]
///     user_id: ForeignKey<i32>,
/// }
///
/// // Use the foreign key to and join the tables together
/// // to get the user posts
/// let user_posts = Users::select()
///     .columns(vec!["Users.name", "Posts.title"])
///     .join(Posts::table())
///     .order_by("name", geekorm::QueryOrder::Asc)
///     .build()
///     .expect("Failed to build query");
///
/// println!("User posts query: {:?}", user_posts);
/// // ...
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct ForeignKey<T>
where
    T: Display + 'static,
{
    pub(crate) value: T,
}

/// Foreign Key as an Integer
pub type ForeignKeyInteger = ForeignKey<i32>;

impl<T> Debug for ForeignKey<T>
where
    T: Debug + Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForeignKey({})", self.value)
    }
}

impl<T> Display for ForeignKey<T>
where
    T: Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for ForeignKey<i32> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl Default for ForeignKey<String> {
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}
impl ForeignKey<i32> {
    /// Create a new foreign key with an integer
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl ForeignKey<String> {
    /// Create a new foreign key with a String
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl From<i32> for ForeignKey<i32> {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl From<String> for ForeignKey<String> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<PrimaryKey<i32>> for ForeignKey<i32> {
    fn from(value: PrimaryKey<i32>) -> Self {
        Self::new(value.value)
    }
}

impl From<PrimaryKey<String>> for ForeignKey<String> {
    fn from(value: PrimaryKey<String>) -> Self {
        Self::new(value.value)
    }
}

impl Serialize for ForeignKeyInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value)
    }
}

impl<'de> Deserialize<'de> for ForeignKeyInteger {
    fn deserialize<D>(deserializer: D) -> Result<ForeignKeyInteger, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = ForeignKeyInteger;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer representing a primary key")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ForeignKeyInteger::from(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ForeignKeyInteger::from(v as i32))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ForeignKeyInteger::from(v as i32))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value.parse::<i32>() {
                    Ok(value) => Ok(ForeignKeyInteger::from(value)),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.parse::<i32>() {
                    Ok(value) => Ok(ForeignKeyInteger::from(value)),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }
        }

        deserializer.deserialize_i32(PrimaryKeyVisitor)
    }
}
