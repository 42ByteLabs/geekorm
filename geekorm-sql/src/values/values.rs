//! # Values
//!
//! A collection of values to be used in SQL queries.
use super::value::Value;

/// A collection of values to be used in SQL queries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Values {
    /// List of values
    pub(crate) values: Vec<(String, Value)>,
}

impl Values {
    /// Create a new instance of Values
    pub fn new() -> Self {
        Values { values: Vec::new() }
    }

    /// Push a value to the list of values
    pub fn push(&mut self, column: String, value: impl Into<Value>) {
        self.values.push((column, value.into()));
    }

    /// Get a value by index from the list of values
    pub fn get(&self, column: &String) -> Option<&Value> {
        self.values
            .iter()
            .find_map(|(c, o)| if c == column { Some(o) } else { None })
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
    pub fn values(&self) -> &Vec<(String, Value)> {
        &self.values
    }
}

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values
            .into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<Value>>()
            .into_iter()
    }
}
