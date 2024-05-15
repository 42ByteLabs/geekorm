//! The Utility module
//!
//! This is where all the utility functions are defined for the GeekORM crate.
//! There are several utility functions that are used in the crate.
//!
//! - Cryptography
//!   - `generate_random_string` - Generate a random string
//!   - `generate_hash` - Generate a hash
//!

/// Cryptography module
pub mod crypto;

#[cfg(feature = "rand")]
pub use crypto::generate_random_string;

#[cfg(feature = "hash")]
pub use crypto::generate_hash;
#[cfg(feature = "hash")]
pub use crypto::verify_hash;
