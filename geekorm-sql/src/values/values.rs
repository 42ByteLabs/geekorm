//! # Values
//!
//! A collection of values to be used in SQL queries.
use super::value::Value;
use crate::ToSql;

/// The different options on how to bind values
///
/// https://sqlite.org/c3ref/bind_blob.html
#[derive(Debug, Default, Clone, PartialEq)]
pub enum ValueBindingMode {
    /// This is using the standard `?`
    #[default]
    Placeholder,
    /// Named value `:VVV`
    Named,
    /// Numeric like `:NNN`
    Numeric,
}

/// Named Value
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NamedValue {
    name: String,
    value: Value,
}

impl NamedValue {
    /// New NamedValue
    pub fn new(name: impl Into<String>, value: impl Into<Value>) -> Self {
        NamedValue {
            name: name.into(),
            value: value.into(),
        }
    }
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get Value
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl ToSql for NamedValue {
    fn to_sql(&self, query: &crate::QueryBuilder) -> Result<String, crate::prelude::Error> {
        let mut stream = String::new();

        // This is to prevent SQL injection attacks
        // SECURITY: Parameterise all the values being passed in
        match query.values.binding_mode {
            ValueBindingMode::Placeholder => {
                stream.push('?');
            }
            ValueBindingMode::Named => {
                stream.push_str(&format!(":{}", self.name()));
            }
            ValueBindingMode::Numeric => {
                let index = query
                    .values
                    .get_index(self.name())
                    .expect("Failed to fetch value index");
                stream.push_str(&format!("?{}", index));
            }
        }

        Ok(stream)
    }
}

/// A collection of values to be used in SQL queries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Values {
    /// List of values
    pub(crate) values: Vec<NamedValue>,
    /// Binding mode that should be used for these particular values
    pub(crate) binding_mode: ValueBindingMode,
}

impl Values {
    /// Create a new instance of Values
    pub fn new() -> Self {
        Values::default()
    }

    /// Push a value to the list of values
    pub fn push(&mut self, column: impl Into<String>, value: impl Into<Value>) {
        self.values
            .push(NamedValue::new(column.into(), value.into()))
    }

    /// Get a value by index from the list of values
    pub fn get(&self, column: &String) -> Option<&Value> {
        self.values.iter().find_map(|nv| {
            if nv.name == *column {
                Some(&nv.value)
            } else {
                None
            }
        })
    }

    /// Length / Count of the values stored
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the values are empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the values
    pub fn values(&self) -> &Vec<NamedValue> {
        &self.values
    }

    /// Gets the index of the column (starts from 1)
    pub fn get_index(&self, column: &str) -> Option<usize> {
        for (index, value) in self.values.iter().enumerate() {
            if value.name == column {
                return Some(index + 1);
            }
        }
        None
    }
}

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values
            .into_iter()
            .map(|v| v.value().clone())
            .collect::<Vec<Value>>()
            .into_iter()
    }
}

impl From<NamedValue> for Value {
    fn from(value: NamedValue) -> Self {
        value.value
    }
}

#[cfg(test)]
mod tests {
    use crate::QueryBuilder;

    use super::*;

    #[test]
    fn test_values_get_index() {
        let mut values = Values::new();
        values.push("id", Value::Integer(1));
        values.push("username", Value::Text("GeekMasher".to_string()));
        values.push("first_name", Value::Text("mathew".to_string()));
        values.push("age", Value::Integer(42)); // I'm not :( 

        assert_eq!(values.len(), 4);
        assert_eq!(values.get_index("id"), Some(1));
        assert_eq!(values.get_index("username"), Some(2));
        assert_eq!(values.get_index("first_name"), Some(3));
        assert_eq!(values.get_index("age"), Some(4));
    }

    #[test]
    fn sqlite_named_value() {
        let mut builder = QueryBuilder::update();
        builder.set_value_mode(ValueBindingMode::Named);

        let named = NamedValue::new("id", Value::Integer(1));
        let query = named.to_sql(&builder).unwrap();
        assert_eq!(query.as_str(), ":id");

        let named = NamedValue::new("username", Value::Text("geekmasher".to_string()));
        let query = named.to_sql(&builder).unwrap();
        assert_eq!(query.as_str(), ":username");

        let named = NamedValue::new("id", Value::Identifier(1));
        let query = named.to_sql(&builder).unwrap();
        assert_eq!(query.as_str(), ":id");
    }
}
