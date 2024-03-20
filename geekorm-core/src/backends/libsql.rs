use crate::backends::Backend;

pub struct LibSqlBackend {}

impl Backend for LibSqlBackend {
    fn execute(&self, query: &str) -> Result<(), crate::Error> {}
}
