//! The Utility module
//!
//! This is where all the utility functions are defined for the GeekORM crate.
//! There are several utility functions that are used in the crate.
//!
//! - Cryptography
//!   - `generate_random_string` - Generate a random string
//!   - `generate_hash` - Generate a hash
//!

/// The Cryptography module
pub mod crypto;
#[cfg(feature = "two-factor-auth")]
pub mod tfa;

#[cfg(feature = "hash")]
pub use crypto::hashing::{generate_hash, verify_hash};
#[cfg(feature = "rand")]
pub use crypto::rand::generate_random_string;

#[cfg(feature = "two-factor-auth")]
pub use tfa::TwoFactorAuth;
