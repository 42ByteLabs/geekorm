use crate::PrimaryKey;
use libsql::Value;

impl From<PrimaryKey> for Value {
    fn from(value: PrimaryKey) -> Self {
        value.value.into()
    }
}

impl From<&PrimaryKey> for Value {
    fn from(value: &PrimaryKey) -> Self {
        value.value.clone().into()
    }
}
