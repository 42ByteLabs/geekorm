//! # Driver implementations

use super::GeekConnector;

pub trait Driver: Sync + Send {
    /// Connect to the database
    fn connect(&self, uri: impl Into<String>) -> Result<Box<dyn GeekConnector>, crate::Error>;
}
