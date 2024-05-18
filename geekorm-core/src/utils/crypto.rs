/// This module contains functions for generating random strings

#[cfg(feature = "rand")]
use rand::Rng;

#[cfg(feature = "hash")]
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
#[cfg(feature = "hash-sha512")]
use sha_crypt::{sha512_check, sha512_simple, Sha512Params};

/// Character set for generating random strings
pub const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

/// Generate a random string of a given length
#[cfg(feature = "rand")]
pub fn generate_random_string(length: usize, prefix: impl Into<String>) -> String {
    let mut rng = rand::thread_rng();
    let mut random_string = String::new();
    // Generate a random string of the given length using uppercase, lowercase and numbers
    for _ in 0..length {
        let random_char = CHARSET[rng.gen_range(0..CHARSET.len())] as char;
        random_string.push(random_char);
    }
    prefix.into() + &random_string
}

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
    Argon2,
    /// SHA512 + Rounds (100k) Hashing Algorithm
    Sha512,
}

impl HashingAlgorithm {
    /// Convert to string slice
    pub fn to_str(&self) -> &str {
        match self {
            HashingAlgorithm::Pbkdf2 => "Pbkdf2",
            HashingAlgorithm::Argon2 => "Argon2",
            HashingAlgorithm::Sha512 => "Sha512",
        }
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

/// Generate a hash for a given string
///
///
#[cfg(feature = "hash")]
pub fn generate_hash(data: String, alg: HashingAlgorithm) -> Result<String, crate::Error> {
    match alg {
        HashingAlgorithm::Pbkdf2 => generate_hash_pdkdf2(data),
        #[cfg(feature = "hash-sha512")]
        HashingAlgorithm::Sha512 => generate_hash_sha512(data),
    }
}

/// Verify a hash for a given string
#[cfg(feature = "hash")]
pub fn verify_hash(
    data: String,
    hash: String,
    alg: HashingAlgorithm,
) -> Result<bool, crate::Error> {
    match alg {
        HashingAlgorithm::Pbkdf2 => verify_hash_pbkdf2(data, hash),
        #[cfg(feature = "hash-sha512")]
        HashingAlgorithm::Sha512 => match sha512_check(data.as_str(), hash.as_str()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        },
    }
}

/// Generate a hash using PBKDF2
///
/// ```rust
/// use geekorm_core::utils::crypto::{generate_hash, HashingAlgorithm};
///
/// let data = "password".to_string();
/// let hash = generate_hash(data, HashingAlgorithm::Pbkdf2);
/// ```
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
