//! # GeekORM SHA512 implementation
//!
//! https://docs.rs/sha-crypt/latest/

use password_hash::phc::SaltString;
use sha_crypt::{Params, PasswordHasher, PasswordVerifier, ShaCrypt};

fn sha512() -> Result<ShaCrypt, crate::Error> {
    // Parameters
    let params = match Params::new(100_000) {
        Ok(p) => p,
        Err(_) => {
            return Err(crate::Error::HashingError(String::from(
                "Error creating params for sha512",
            )));
        }
    };
    Ok(ShaCrypt::new(sha_crypt::Algorithm::Sha512Crypt, params))
}

/// Generate a hash using SHA512 + Rounds
pub(crate) fn generate_hash_sha512(data: String) -> Result<String, crate::Error> {
    // Salt
    let salt = SaltString::generate();
    let hasher = sha512()?;

    match hasher.hash_password_with_salt(data.as_bytes(), &salt.to_salt()) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err(crate::Error::HashingError(
            "Error hashing password using SHA512".to_string(),
        )),
    }
}

pub(crate) fn verify_hash_sha512(data: String, hash: String) -> Result<bool, crate::Error> {
    let hasher = sha512()?;
    match hasher.verify_password(data.as_bytes(), hash.as_str()) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing_sha512() {
        let clear_text = "super-secret-geekorm-password";
        let cipher_text = generate_hash_sha512(clear_text.to_string()).unwrap();

        // Split out the PHC and make sure it follows the spec
        let comps: Vec<&str> = cipher_text.split('$').collect();
        assert_eq!(comps.len(), 5);
        assert_eq!(comps.get(1).unwrap(), &"6");

        let params: Vec<&str> = comps.get(2).unwrap().split(',').collect();
        for param in params.iter() {
            if param.starts_with("rounds=") {
                assert_eq!(param, &"rounds=100000");
            }
        }

        let salt = comps.get(2).unwrap();
        assert!(salt.len() > 12);

        let result = verify_hash_sha512(clear_text.to_string(), cipher_text).unwrap();
        assert_eq!(result, true);
    }
}
