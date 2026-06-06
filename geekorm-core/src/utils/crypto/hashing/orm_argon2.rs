//! # GeekORM Argon2 implementation
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Generate a hash using Argon2
pub(crate) fn generate_hash_argon2(data: String) -> Result<String, crate::Error> {
    // Salt
    let salt = SaltString::generate(&mut OsRng);
    // Hash
    let argon2 = Argon2::default();

    match argon2.hash_password(data.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(crate::Error::HashingError(format!(
            "Error hashing password: {}",
            e
        ))),
    }
}

pub(crate) fn verify_hash_argon2(data: String, hash: String) -> Result<bool, crate::Error> {
    let hasher = PasswordHash::new(&hash)
        .map_err(|e| crate::Error::HashingError(format!("Error parsing password hash: {}", e)))?;
    match Argon2::default().verify_password(data.as_bytes(), &hasher) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasing_argon2() {
        let clear_text = "super-secret-geekorm-password";
        let cipher_text = generate_hash_argon2(clear_text.to_string()).unwrap();

        // Split out the PHC and make sure it follows the spec
        let comps: Vec<&str> = cipher_text.split('$').collect();
        assert_eq!(comps.len(), 6);
        assert_eq!(comps.get(1).unwrap(), &"argon2id");

        let params: Vec<&str> = comps.get(3).unwrap().split(',').collect();
        for param in params.iter() {
            if param.starts_with("i=") {
                assert_eq!(param, &"i=210000");
            } else if param.starts_with("l=") {
                assert_eq!(param, &"l=32");
            }
        }

        let salt = comps.get(4).unwrap();
        assert!(salt.len() > 12);

        let result = verify_hash_argon2(clear_text.to_string(), cipher_text).unwrap();
        assert_eq!(result, true);
    }
}
