//! # Values
use serde::{Deserialize, Serialize, Serializer};
use std::{fmt::Display, str};

pub mod time;
pub mod value;
pub mod values;

#[cfg(feature = "chrono")]
pub(crate) mod chrono;
#[cfg(feature = "semver")]
pub(crate) mod semver;
#[cfg(feature = "url")]
pub(crate) mod url;
#[cfg(feature = "uuid")]
pub(crate) mod uuid;

pub use value::Value;
pub use values::Values;
