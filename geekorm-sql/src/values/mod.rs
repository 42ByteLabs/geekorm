//! # Values
use std::{fmt::Display, str};

use serde::{Deserialize, Serialize, Serializer};

pub mod time;
pub mod value;
pub mod values;

#[cfg(feature = "uuid")]
pub(crate) mod uuid;

pub use value::Value;
