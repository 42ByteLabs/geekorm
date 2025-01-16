//! # Postgres deserialization
use crate::Value;
use bytes::BytesMut;
use serde::de::{Error, MapAccess, Visitor};
use serde::Deserialize;
use serde::{de::value::Error as DeError, Deserializer};
use tokio_postgres::types::ToSql;

struct PostgresRow<'de> {
    row: &'de tokio_postgres::Row,
}

impl<'de> Deserializer<'de> for PostgresRow<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(DeError::custom("Expected a struct"))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct RowMap<'a> {
            row: &'a tokio_postgres::Row,
            index: std::ops::Range<usize>,
            value: Option<&'a tokio_postgres::types::Type>,
        }

        impl<'de> MapAccess<'de> for RowMap<'de> {
            type Error = DeError;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: serde::de::DeserializeSeed<'de>,
            {
                match self.index.next() {
                    Some(index) => {
                        let column = self
                            .row
                            .columns()
                            .get(index)
                            .ok_or(DeError::custom("Invalid column index"))?;

                        self.value = Some(column.type_());

                        seed.deserialize(&mut *self).map(Some).transpose()
                    }
                    None => Ok(None),
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                let value = self.value.ok_or(DeError::custom("No value"))?;

                seed.deserialize(value.into_deserializer())
            }
        }

        visitor.visit_map(RowMap {
            row: self.row,
            index: 0..self.row.len(),
            value: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

/// Convert a row to a struct
pub(crate) fn from_row<'de, T: Deserialize<'de>>(
    row: &'de tokio_postgres::Row,
) -> Result<T, DeError> {
    let deserializer = PostgresRow { row };
    T::deserialize(deserializer)
}

impl Value {
    pub(super) fn to_postgres_type(&self) -> tokio_postgres::types::Type {
        match self {
            Value::Boolean(_) => tokio_postgres::types::Type::BOOL,
            Value::Text(_) => tokio_postgres::types::Type::TEXT,
            Value::Integer(_) => tokio_postgres::types::Type::INT8,
            Value::Identifier(_) => tokio_postgres::types::Type::TEXT,
            Value::Blob(_) => tokio_postgres::types::Type::BYTEA,
            Value::Json(_) => tokio_postgres::types::Type::JSONB,
            _ => unimplemented!(),
        }
    }
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        match self {
            Value::Null => Ok(tokio_postgres::types::IsNull::Yes),
            Value::Boolean(value) => value.to_sql(ty, out),
            Value::Text(value) => value.to_sql(ty, out),
            Value::Integer(value) => value.to_sql(ty, out),
            Value::Identifier(value) => value.to_sql(ty, out),
            Value::Blob(value) => value.to_sql(ty, out),
            Value::Json(value) => value.to_sql(ty, out),
        }
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        ty.name() == "jsonb"
    }

    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Value::Null => Ok(tokio_postgres::types::IsNull::Yes),
            Value::Boolean(value) => value.to_sql_checked(ty, out),
            Value::Text(value) => value.to_sql_checked(ty, out),
            Value::Integer(value) => value.to_sql_checked(ty, out),
            Value::Identifier(value) => value.to_sql_checked(ty, out),
            Value::Blob(value) => value.to_sql_checked(ty, out),
            Value::Json(value) => value.to_sql_checked(ty, out),
        }
    }
}
