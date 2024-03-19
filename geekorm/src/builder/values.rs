#[derive(Debug, Clone, Default)]
pub struct Values {
    pub values: Vec<Value>,
}

impl Values {
    pub fn new() -> Self {
        Values { values: Vec::new() }
    }
}

impl Iterator for Values {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}

impl Values {
    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Text(String),
    Integer(i32),
}

impl Default for Value {
    fn default() -> Self {
        Value::Text(String::new())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value as i32)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Integer(value as i32)
    }
}
