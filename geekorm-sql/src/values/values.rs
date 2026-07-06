//! # Values
//!
//! A collection of values to be used in SQL queries.
use super::value::Value;

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
    pub fn value(&self) -> Value {
        self.value.clone()
    }
}

/// A collection of values to be used in SQL queries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Values {
    /// List of values
    pub(crate) values: Vec<NamedValue>,
}

impl Values {
    /// Create a new instance of Values
    pub fn new() -> Self {
        Values { values: Vec::new() }
    }

    /// Push a value to the list of values
    pub fn push(&mut self, column: String, value: impl Into<Value>) {
        self.values.push(NamedValue::new(column, value.into()))
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
}

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values
            .into_iter()
            .map(|v| v.value())
            .collect::<Vec<Value>>()
            .into_iter()
    }
}

impl From<NamedValue> for Value {
    fn from(value: NamedValue) -> Self {
        value.value
    }
}
