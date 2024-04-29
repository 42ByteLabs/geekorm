/// This module contains functions for generating random strings

#[cfg(feature = "rand")]
use rand::Rng;

#[cfg(feature = "hash")]
use pbkdf2::{
    password_hash::{rand_core::OsRng, SaltString},
    pbkdf2_hmac,
};
#[cfg(feature = "hash")]
use sha2::Sha256;

/// Character set for generating random strings
pub const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

/// Generate a random string of a given length
#[cfg(feature = "rand")]
pub fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut random_string = String::new();
    // Generate a random string of the given length using uppercase, lowercase and numbers
    for _ in 0..length {
        let random_char = CHARSET[rng.gen_range(0..CHARSET.len())] as char;
        random_string.push(random_char);
    }
    random_string
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

impl From<&str> for HashingAlgorithm {
    fn from(s: &str) -> Self {
        match s {
            "Pbkdf2" => HashingAlgorithm::Pbkdf2,
            _ => HashingAlgorithm::Pbkdf2,
        }
    }
}

/// Generate a hash for a given string
///
///
#[cfg(feature = "hash")]
pub fn generate_hash(data: String, alg: HashingAlgorithm) -> String {
    match alg {
        HashingAlgorithm::Pbkdf2 => generate_hash_pdkdf2(data),
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
pub(crate) fn generate_hash_pdkdf2(data: String) -> String {
    // Salt
    let salt = SaltString::generate(&mut OsRng);
    // Hash
    let mut hash = [0u8; 20];

    pbkdf2_hmac::<Sha256>(
        data.as_bytes(),
        &salt.as_str().as_bytes(),
        600_000,
        &mut hash,
    );

    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "rand")]
    fn test_generate_random_string() {
        let random_string = generate_random_string(10);
        assert_eq!(random_string.len(), 10);
    }
}
