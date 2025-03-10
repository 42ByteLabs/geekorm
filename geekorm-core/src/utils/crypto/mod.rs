//! This module contains functions for generating random strings

use std::fmt::Display;

/// Random number generator
#[cfg(feature = "rand")]
pub mod rand;

/// Hashing module
#[cfg(feature = "hash")]
pub mod hashing;

#[cfg(feature = "hash")]
use crate::utils::crypto::hashing::{generate_hash, verify_hash};

/// Hashing algorithms
#[derive(Default, Clone, Debug)]
pub enum HashingAlgorithm {
    /// PBKDF2 Hashing Algorithm
    ///
    /// This is the default hashing algorithm and is the most secure of all
    /// supported algorithms.
    #[default]
    Pbkdf2,
    /// Argon2 Hashing Algorithm
    ///
    /// Argon2id v19 + Salt
    #[cfg(feature = "hash-argon2")]
    Argon2,
    /// SHA512 + Rounds (100k) Hashing Algorithm
    ///
    /// Weakest of all supported algorithms but fastest
    #[cfg(feature = "hash-sha512")]
    Sha512,
}

impl HashingAlgorithm {
    /// Convert to string slice
    pub fn to_str(&self) -> &str {
        match self {
            HashingAlgorithm::Pbkdf2 => "Pbkdf2",
            #[cfg(feature = "hash-argon2")]
            HashingAlgorithm::Argon2 => "Argon2",
            #[cfg(feature = "hash-sha512")]
            HashingAlgorithm::Sha512 => "Sha512",
        }
    }

    /// Generate a hash using the selected algorithm
    #[cfg(feature = "hash")]
    pub fn generate_hash(&self, data: String) -> Result<String, crate::Error> {
        generate_hash(data, self.clone())
    }

    /// Verify a hash using the selected algorithm
    #[cfg(feature = "hash")]
    pub fn verify_hash(&self, data: String, hash: String) -> Result<bool, crate::Error> {
        verify_hash(data, hash, self.clone())
    }
}

impl TryFrom<&str> for HashingAlgorithm {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "pbkdf2" => Ok(HashingAlgorithm::Pbkdf2),
            #[cfg(feature = "hash-argon2")]
            "argon2" => Ok(HashingAlgorithm::Argon2),
            #[cfg(feature = "hash-sha512")]
            "sha512" => Ok(HashingAlgorithm::Sha512),
            _ => Err(crate::Error::HashingError(format!(
                "Invalid hashing algorithm: {}",
                value
            ))),
        }
    }
}

impl TryFrom<&String> for HashingAlgorithm {
    type Error = crate::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for HashingAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashingAlgorithm({})", self.to_str())
    }
}
