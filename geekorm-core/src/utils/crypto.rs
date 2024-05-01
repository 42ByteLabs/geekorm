/// This module contains functions for generating random strings

#[cfg(feature = "rand")]
use rand::Rng;

#[cfg(feature = "hash")]
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

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
    /// PBKDF2
    #[default]
    Pbkdf2,
}

impl HashingAlgorithm {
    /// Convert to string slice
    pub fn to_str(&self) -> &str {
        match self {
            HashingAlgorithm::Pbkdf2 => "Pbkdf2",
        }
    }
}

impl TryFrom<&str> for HashingAlgorithm {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "pbkdf2" => Ok(HashingAlgorithm::Pbkdf2),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "rand")]
    fn test_generate_random_string() {
        let random_string = generate_random_string(10, "");
        assert_eq!(random_string.len(), 10);
    }
}
