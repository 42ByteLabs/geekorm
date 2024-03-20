#[cfg(feature = "libsql")]
pub(crate) mod libsql;

use crate::Error;

pub(crate) trait Connection {
    fn execute(&self, query: &str) -> Result<(), Error>;
}

// pub(crate) struct Backend {}
