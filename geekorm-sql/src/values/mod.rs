//! # Values
use std::{fmt::Display, str};

use serde::{Deserialize, Serialize, Serializer};

pub mod value;
pub mod values;

#[cfg(feature = "chrono")]
pub(crate) mod datetime;
#[cfg(feature = "semver")]
pub(crate) mod semver;
#[cfg(feature = "uuid")]
pub(crate) mod uuid;

pub use value::Value;
