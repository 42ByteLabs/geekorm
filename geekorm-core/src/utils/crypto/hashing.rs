#[cfg(feature = "hash-argon2")]
use argon2::Argon2;
#[cfg(feature = "hash-pbkdf2")]
use pbkdf2::Pbkdf2;
#[cfg(feature = "hash-sha512")]
use sha_crypt::{sha512_check, sha512_simple, Sha512Params};
// Password Hashing Library
use password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::utils::crypto::HashingAlgorithm;

/// Generate a hash for a given string
#[allow(unreachable_patterns)]
pub fn generate_hash(data: String, alg: HashingAlgorithm) -> Result<String, crate::Error> {
    match alg {
        #[cfg(feature = "hash-pbkdf2")]
        HashingAlgorithm::Pbkdf2 => generate_hash_pdkdf2(data),
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
        HashingAlgorithm::Pbkdf2 => verify_hash_pbkdf2(data, hash),
        #[cfg(feature = "hash-argon2")]
        HashingAlgorithm::Argon2 => {
            let hasher = PasswordHash::new(&hash)?;
            match Argon2::default().verify_password(data.as_bytes(), &hasher) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        #[cfg(feature = "hash-sha512")]
        HashingAlgorithm::Sha512 => match sha512_check(data.as_str(), hash.as_str()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        },
        _ => Err(crate::Error::HashingError(
            "Invalid hashing algorithm".to_string(),
        )),
    }
}

/// Generate a hash using PBKDF2
#[cfg(feature = "hash")]
pub(crate) fn generate_hash_pdkdf2(data: String) -> Result<String, crate::Error> {
    // Salt
    let salt = SaltString::generate(&mut OsRng);
    // Hash
    match Pbkdf2.hash_password(data.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(crate::Error::HashingError(format!(
            "Error hashing password: {}",
            e
        ))),
    }
}

#[cfg(feature = "hash")]
pub(crate) fn verify_hash_pbkdf2(data: String, hash: String) -> Result<bool, crate::Error> {
    let parsed_hash = match PasswordHash::new(&hash) {
        Ok(h) => h,
        Err(e) => {
            return Err(crate::Error::HashingError(format!(
                "Error parsing password hash: {}",
                e
            )))
        }
    };

    match Pbkdf2.verify_password(data.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generate a hash using Argon2
#[cfg(feature = "hash-argon2")]
pub(crate) fn generate_hash_argon2(data: String) -> Result<String, crate::Error> {
    // Salt
    let salt = SaltString::generate(&mut OsRng);
    // Hash
    let argon2 = Argon2::default();
    match argon2.hash_password(data.as_bytes(), salt.as_ref()) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(crate::Error::HashingError(format!(
            "Error hashing password: {}",
            e
        ))),
    }
}

/// Generate a hash using SHA512 + Rounds
#[cfg(feature = "hash-sha512")]
pub(crate) fn generate_hash_sha512(data: String) -> Result<String, crate::Error> {
    let params = match Sha512Params::new(100_000) {
        Ok(p) => p,
        Err(_) => {
            return Err(crate::Error::HashingError(String::from(
                "Error creating params for sha512",
            )))
        }
    };
    match sha512_simple(data.as_str(), &params) {
        Ok(hash) => Ok(hash),
        Err(_) => Err(crate::Error::HashingError(format!(
            "Error hashing password using SHA512",
        ))),
    }
}
