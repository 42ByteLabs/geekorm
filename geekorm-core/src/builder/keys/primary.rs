use core::fmt;
use std::fmt::{Debug, Display};

use serde::{de::Visitor, Deserialize, Serialize, Serializer};
use uuid::Uuid;

use crate::ToSqlite;

/// Primary Key Type
///
/// The Primary Key is a column in a Table used to uniquely identify a row.
///
/// In GeekORM, it can be an `i32` (default), a `String`, or a `Uuid`.
///
/// # Example
///
/// Here is an example of how to use the PrimaryKey struct with an integer
///
/// ```rust
/// use geekorm::PrimaryKey;
///
/// struct User {
///    pub id: PrimaryKey<i32>,
///    pub username: String,
/// }
///
/// let user = User {
///     id: PrimaryKey::from(1),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.id, PrimaryKey::from(1));
/// # assert_eq!(user.username, String::from("JohnDoe"));
/// ```
///
/// Here is an example of how to use the PrimaryKey struct with a String
///
/// ```rust
/// use geekorm::PrimaryKey;
///
/// struct User {
///     pub id: PrimaryKey<String>,
///     pub username: String,
/// }
///
/// let user = User {
///     id: PrimaryKey::from("1"),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.id, PrimaryKey::from(String::from("1")));
/// # assert_eq!(user.username, String::from("JohnDoe"));
/// ```
///
/// Here is an example of how to use the PrimaryKey struct with a Uuid.
///
/// Note: This requires the `uuid` feature to be enabled.
///
/// ```rust
/// use geekorm::PrimaryKeyUuid;
///
/// struct User {
///     pub id: PrimaryKeyUuid,
///     pub username: String,
/// }
///
/// let user = User {
///     id: PrimaryKeyUuid::from(uuid::Uuid::new_v4()),
///     username: String::from("JohnDoe")
/// };
/// # assert_eq!(user.username, String::from("JohnDoe"));
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct PrimaryKey<T>
where
    T: Display + 'static,
{
    pub(crate) value: T,
}

impl<T> Debug for PrimaryKey<T>
where
    T: Debug + Display + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimaryKey({})", self.value)
    }
}

impl PrimaryKey<i32> {
    /// Create a new primary key with an integer
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl PrimaryKey<String> {
    /// Create a new primary key with a String
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// Primary Key as an Integer
pub type PrimaryKeyInteger = PrimaryKey<i32>;

impl Default for PrimaryKeyInteger {
    fn default() -> Self {
        PrimaryKey { value: 0 }
    }
}

/// Primary Key as a Uuid
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

impl From<i32> for PrimaryKey<i32> {
    fn from(value: i32) -> Self {
        PrimaryKey { value }
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

impl From<&str> for PrimaryKey<String> {
    fn from(value: &str) -> Self {
        PrimaryKey {
            value: String::from(value),
        }
    }
}

impl From<String> for PrimaryKeyUuid {
    fn from(value: String) -> Self {
        PrimaryKeyUuid {
            value: Uuid::parse_str(&value).unwrap(),
        }
    }
}

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

impl From<PrimaryKeyInteger> for i32 {
    fn from(value: PrimaryKeyInteger) -> Self {
        value.value
    }
}

impl Serialize for PrimaryKeyInteger {
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

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v as i32))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PrimaryKeyInteger::from(v as i32))
            }

            fn visit_str<E>(self, value: &str) -> Result<PrimaryKeyInteger, E>
            where
                E: serde::de::Error,
            {
                match value.parse::<i32>() {
                    Ok(value) => Ok(PrimaryKeyInteger::from(value)),
                    Err(_) => Err(serde::de::Error::custom("Invalid integer value")),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.parse::<i32>() {
                    Ok(value) => Ok(PrimaryKeyInteger::from(value)),
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
