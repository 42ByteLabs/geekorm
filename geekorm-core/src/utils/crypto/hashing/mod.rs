//! # Password Hashing
//!

#[cfg(feature = "hash-argon2")]
pub(crate) mod orm_argon2;
#[cfg(feature = "hash-pbkdf2")]
pub(crate) mod orm_pbkdf2;
#[cfg(feature = "hash-sha512")]
pub(crate) mod orm_sha512;

use crate::utils::crypto::HashingAlgorithm;
#[cfg(feature = "hash-argon2")]
use orm_argon2::{generate_hash_argon2, verify_hash_argon2};
#[cfg(feature = "hash-pbkdf2")]
use orm_pbkdf2::{generate_hash_pbkdf2, verify_hash_pbkdf2};
#[cfg(feature = "hash-sha512")]
use orm_sha512::{generate_hash_sha512, verify_hash_sha512};

/// Generate a hash for a given string
#[allow(unreachable_patterns)]
pub fn generate_hash(data: String, alg: HashingAlgorithm) -> Result<String, crate::Error> {
    match alg {
        #[cfg(feature = "hash-pbkdf2")]
        HashingAlgorithm::Pbkdf2 => generate_hash_pbkdf2(data),
        #[cfg(feature = "hash-argon2")]
        HashingAlgorithm::Argon2 => generate_hash_argon2(data),
        #[cfg(feature = "hash-sha512")]
        HashingAlgorithm::Sha512 => generate_hash_sha512(data),
        _ => Err(crate::Error::HashingError(
            "Invalid hashing algorithm".to_string(),
        )),
    }
}

/// Verify a hash for a given string
///
/// ```rust
/// use geekorm_core::utils::{verify_hash, generate_hash, crypto::HashingAlgorithm};
///
/// let data = "password".to_string();
/// let hash = generate_hash(data.clone(), HashingAlgorithm::Pbkdf2).unwrap();
///
/// if verify_hash(data, hash, HashingAlgorithm::Pbkdf2).unwrap() {
///     println!("Password is correct");
/// } else {
///     println!("Password is incorrect");
/// }
/// ```
#[cfg(feature = "hash")]
pub fn verify_hash(
    data: String,
    hash: String,
    alg: HashingAlgorithm,
) -> Result<bool, crate::Error> {
    match alg {
        #[cfg(feature = "hash-pbkdf2")]
        HashingAlgorithm::Pbkdf2 => verify_hash_pbkdf2(data, hash),
        #[cfg(feature = "hash-argon2")]
        HashingAlgorithm::Argon2 => verify_hash_argon2(data, hash),
        #[cfg(feature = "hash-sha512")]
        HashingAlgorithm::Sha512 => verify_hash_sha512(data, hash),
    }
}
