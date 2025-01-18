//! # Primary Key
//!
//! Primary Keys are used to uniquely identify a row in a table.
//! GeekORM supports three primary key types:
//!
//! - `PrimaryKeyInteger` (default)
//! - `PrimaryKeyString`
//! - `PrimaryKeyUuid` (requires the `uuid` feature to be enabled)
//!
//! # Standard Example
//!
//! Here is an example of how to use the PrimaryKey.
//!
//! ```rust
//! use geekorm::prelude::*;
//!
//! #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
//! pub struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     pub id: PrimaryKeyInteger,
//!     #[geekorm(unique)]
//!     pub username: String,
//! }
//!
//! let user = Users {
//!     id: PrimaryKey::from(1),
//!     username: String::from("JohnDoe")
//! };
//! # assert_eq!(Users::primary_key(), "id");
//! # assert_eq!(user.id.clone(), PrimaryKey::from(1));
//! # assert_eq!(user.username.clone(), String::from("JohnDoe"));
//! ```
//!
//! # String Example
//!
//! Here is an example of how to use the PrimaryKey struct with a String as the primary key.
//!
//! ```rust
//! use geekorm::prelude::*;
//!
//! #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
//! pub struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     pub id: PrimaryKeyString,
//!     #[geekorm(unique)]
//!     pub username: String,
//! }
//!
//! let user = Users {
//!     id: PrimaryKey::from("1"),
//!     username: String::from("JohnDoe")
//! };
//! # assert_eq!(user.id.clone(), PrimaryKey::from("1"));
//! # assert_eq!(user.username.clone(), String::from("JohnDoe"));
//! ```
//!
//! # Uuid Example
//!
//! With the `uuid` feature enabled, you can use the `PrimaryKeyUuid` struct to use a
//! Uuid as the primary key.
//!
//! ```rust
//! use geekorm::prelude::*;
//!
//!
//! #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
//! pub struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     pub id: PrimaryKeyUuid,
//!     #[geekorm(unique)]
//!     pub username: String,
//! }
//!
//! let new_uuid = uuid::Uuid::new_v4();
//! let user = Users {
//!     id: PrimaryKeyUuid::from(new_uuid),
//!     username: String::from("JohnDoe")
//! };
//! # assert_eq!(user.username.clone(), String::from("JohnDoe"));
//! # assert_eq!(user.id.clone(), PrimaryKeyUuid::from(new_uuid));
//! ```
//!
use core::fmt;
use std::fmt::{Debug, Display};

use serde::{de::Visitor, Deserialize, Serialize, Serializer};
#[cfg(feature = "uuid")]
use uuid::Uuid;

use crate::ToSqlite;

/// Primary Key Type
///
/// The Primary Key is a column in a Table used to uniquely identify a row.
///
/// In GeekORM, it can be an `u64` (default), a `String`, or a `Uuid`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PrimaryKey<T>
where
    T: serde::Serialize + 'static,
{
    pub(crate) value: T,
}

impl<T> PrimaryKey<T>
where
    T: serde::Serialize + 'static,
{
    /// Get the Primary Key value
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> Debug for PrimaryKey<T>
where
    T: serde::Serialize + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimaryKey({:?})", self.value)
    }
}

impl<T> Display for PrimaryKey<T>
where
    T: serde::Serialize + Display + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PrimaryKey<u64> {
    /// Create a new primary key with an integer
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

impl PrimaryKey<String> {
    /// Create a new primary key with a String
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// Primary Key as an Integer (u64)
///
/// This is the default primary key type for GeekORM.
///
/// ```rust
/// use geekorm::prelude::*;
///
/// #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyInteger,
///     pub username: String,
/// }
///
/// let user = Users {
///     id: PrimaryKeyInteger::from(1),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.id.clone(), PrimaryKeyInteger::from(1));
/// # assert_eq!(user.username.clone(), String::from("JohnDoe"));
/// ```
pub type PrimaryKeyInteger = PrimaryKey<u64>;

impl Default for PrimaryKeyInteger {
    fn default() -> Self {
        PrimaryKey { value: 0 }
    }
}

/// This is the old primary key type for GeekORM.
///
/// This is used for backwards compatibility.
pub(crate) type PrimaryKeyIntegerOld = PrimaryKey<i32>;

impl Default for PrimaryKeyIntegerOld {
    fn default() -> Self {
        PrimaryKey { value: 0 }
    }
}

/// PrimaryKeyString (alias) is a Primary Key as a String type
///
/// ```rust
/// use geekorm::prelude::*;
///
/// #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyString,
///     pub username: String,
/// }
///
/// let user = Users {
///     id: PrimaryKeyString::from("1"),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.id.clone(), PrimaryKeyString::from("1"));
/// # assert_eq!(user.username.clone(), String::from("JohnDoe"));
/// ```
pub type PrimaryKeyString = PrimaryKey<String>;

impl Default for PrimaryKeyString {
    fn default() -> Self {
        PrimaryKey {
            value: String::from(""),
        }
    }
}

/// PrimaryKeyUuid (alias) is a Primary Key as a Uuid type
///
/// Note: This requires the `uuid` feature to be enabled.
///
/// ```rust
/// use geekorm::prelude::*;
///
/// #[derive(Table, Clone, Default, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyUuid,
///     pub username: String,
/// }
///
/// let new_uuid = uuid::Uuid::new_v4();
/// let user = Users {
///     id: PrimaryKeyUuid::from(new_uuid),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.username.clone(), String::from("JohnDoe"));
/// # assert_eq!(user.id.clone(), PrimaryKeyUuid::from(new_uuid));
/// ```
#[cfg(feature = "uuid")]
pub type PrimaryKeyUuid = PrimaryKey<Uuid>;

#[cfg(feature = "uuid")]
impl Default for PrimaryKeyUuid {
    fn default() -> Self {
        PrimaryKey {
            value: Uuid::new_v4(),
        }
    }
}

#[cfg(feature = "uuid")]
impl PrimaryKeyUuid {
    /// Create a new primary key with a Uuid
    pub fn new(value: Uuid) -> Self {
        Self { value }
    }
}

#[cfg(feature = "uuid")]
impl From<Uuid> for PrimaryKeyUuid {
    fn from(value: Uuid) -> Self {
        PrimaryKeyUuid::new(value)
    }
}

impl ToSqlite for PrimaryKey<String> {
    fn on_create(&self, _query: &crate::QueryBuilder) -> Result<String, crate::Error> {
        Ok(String::from("PRIMARY KEY"))
    }
}

impl From<u64> for PrimaryKeyInteger {
    fn from(value: u64) -> Self {
        PrimaryKey { value }
    }
}
impl From<i64> for PrimaryKeyInteger {
    fn from(value: i64) -> Self {
        PrimaryKey {
            value: value as u64,
        }
    }
}

/// This is to make sure we are backwards compatible
impl From<i32> for PrimaryKeyInteger {
    fn from(value: i32) -> Self {
        PrimaryKey {
            value: value as u64,
        }
    }
}
impl From<u32> for PrimaryKeyInteger {
    fn from(value: u32) -> Self {
        PrimaryKey {
            value: value as u64,
        }
    }
}

impl From<i32> for PrimaryKeyIntegerOld {
    fn from(value: i32) -> Self {
        PrimaryKey { value }
    }
}
impl From<u32> for PrimaryKeyIntegerOld {
    fn from(value: u32) -> Self {
        PrimaryKey {
            value: value as i32,
        }
    }
}

impl From<String> for PrimaryKeyInteger {
    fn from(value: String) -> Self {
        PrimaryKey {
            value: value.parse().unwrap(),
        }
    }
}

impl From<&str> for PrimaryKeyInteger {
    fn from(value: &str) -> Self {
        PrimaryKey {
            value: value.parse().unwrap(),
        }
    }
}

impl From<String> for PrimaryKey<String> {
    fn from(value: String) -> Self {
        PrimaryKey { value }
    }
}

impl From<&String> for PrimaryKey<String> {
    fn from(value: &String) -> Self {
        PrimaryKey {
            value: value.clone(),
        }
    }
}

impl From<&str> for PrimaryKey<String> {
    fn from(value: &str) -> Self {
        PrimaryKey {
            value: String::from(value),
        }
    }
}

#[cfg(feature = "uuid")]
impl From<String> for PrimaryKeyUuid {
    fn from(value: String) -> Self {
        PrimaryKeyUuid {
            value: Uuid::parse_str(&value).unwrap(),
        }
    }
}

#[cfg(feature = "uuid")]
impl From<&str> for PrimaryKeyUuid {
    fn from(value: &str) -> Self {
        PrimaryKeyUuid {
            value: Uuid::parse_str(value).unwrap(),
        }
    }
}

impl From<PrimaryKey<String>> for String {
    fn from(value: PrimaryKey<String>) -> Self {
        value.value
    }
}

impl From<PrimaryKeyInteger> for u64 {
    fn from(value: PrimaryKeyInteger) -> Self {
        value.value
    }
}
/// This is to make sure we are backwards compatible
impl From<PrimaryKeyInteger> for i32 {
    fn from(value: PrimaryKeyInteger) -> Self {
        value.value as i32
    }
}
impl From<PrimaryKeyIntegerOld> for i32 {
    fn from(value: PrimaryKeyIntegerOld) -> Self {
        value.value
    }
}
impl From<&PrimaryKeyIntegerOld> for i32 {
    fn from(value: &PrimaryKeyIntegerOld) -> Self {
        value.value
    }
}
impl From<PrimaryKeyIntegerOld> for u64 {
    fn from(value: PrimaryKeyIntegerOld) -> Self {
        value.value as u64
    }
}
impl From<&PrimaryKeyIntegerOld> for u64 {
    fn from(value: &PrimaryKeyIntegerOld) -> Self {
        value.value as u64
    }
}

impl Serialize for PrimaryKeyInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.value)
    }
}

impl Serialize for PrimaryKey<i32> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value)
    }
}

impl<'de> Deserialize<'de> for PrimaryKeyInteger {
    fn deserialize<D>(deserializer: D) -> Result<PrimaryKeyInteger, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = PrimaryKeyInteger;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer representing a primary key")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v))
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v as u64))
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKeyInteger, E>
            where
                E: serde::de::Error,
            {
                match value.parse::<u64>() {
                    Ok(value) => Ok(PrimaryKeyInteger::from(value)),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.parse::<u64>() {
                    Ok(value) => Ok(PrimaryKeyInteger::from(value)),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }
        }

        deserializer.deserialize_u64(PrimaryKeyVisitor)
    }
}

/// For backwards compatibility
impl<'de> Deserialize<'de> for PrimaryKeyIntegerOld {
    fn deserialize<D>(deserializer: D) -> Result<PrimaryKeyIntegerOld, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = PrimaryKeyIntegerOld;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer representing a primary key")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyIntegerOld { value: v })
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyIntegerOld { value: v as i32 })
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyIntegerOld { value: v as i32 })
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyIntegerOld { value: v as i32 })
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKeyIntegerOld, E>
            where
                E: serde::de::Error,
            {
                match value.parse::<i32>() {
                    Ok(value) => Ok(PrimaryKeyIntegerOld { value }),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.parse::<u64>() {
                    Ok(value) => Ok(PrimaryKeyIntegerOld {
                        value: value as i32,
                    }),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }
        }

        deserializer.deserialize_i32(PrimaryKeyVisitor)
    }
}

impl Serialize for PrimaryKey<String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for PrimaryKey<String> {
    fn deserialize<D>(deserializer: D) -> Result<PrimaryKey<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = PrimaryKey<String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a primary key")
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKey<String>, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
            fn visit_string<E>(self, value: String) -> Result<PrimaryKey<String>, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
        }

        deserializer.deserialize_str(PrimaryKeyVisitor)
    }
}

#[cfg(feature = "uuid")]
impl Serialize for PrimaryKey<Uuid> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.value.to_string().as_str())
    }
}

#[cfg(feature = "uuid")]
impl<'de> Deserialize<'de> for PrimaryKey<Uuid> {
    fn deserialize<D>(deserializer: D) -> Result<PrimaryKeyUuid, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PrimaryKeyVisitor;

        impl<'de> Visitor<'de> for PrimaryKeyVisitor {
            type Value = PrimaryKeyUuid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a primary key")
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKeyUuid, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
            fn visit_string<E>(self, value: String) -> Result<PrimaryKeyUuid, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKey::from(value))
            }
        }

        deserializer.deserialize_str(PrimaryKeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_key_string() {
        let pk = PrimaryKey::<String>::new(String::from("1"));
        let pk_json = serde_json::to_string(&pk).unwrap();
        assert_eq!(pk_json, "\"1\"");

        let pk_deserialized: PrimaryKey<String> = serde_json::from_str(&pk_json).unwrap();
        assert_eq!(pk, pk_deserialized);
    }

    #[test]
    fn test_primary_key_i32() {
        let pk = PrimaryKeyInteger::new(1);
        let pk_json = serde_json::to_string(&pk).unwrap();

        assert_eq!(pk_json, "1");

        let pk_deserialized: PrimaryKeyInteger = serde_json::from_str(&pk_json).unwrap();
        assert_eq!(pk, pk_deserialized);
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn test_primary_key_uuid() {
        let id = Uuid::new_v4();

        let pk = PrimaryKeyUuid::new(id);
        let pk_json = serde_json::to_string(&pk).unwrap();

        let pk_deserialized: PrimaryKeyUuid = serde_json::from_str(&pk_json).unwrap();
        assert_eq!(pk, pk_deserialized);
    }
}
