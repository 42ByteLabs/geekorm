//! # GeekORM PBKDF2 implementation
use password_hash::phc::SaltString;
use pbkdf2::{
    Params, Pbkdf2,
    password_hash::{PasswordHasher, PasswordVerifier},
    phc::PasswordHash,
};

/// Generate a hash using PBKDF2
///
/// Uses secure defaults from OWASP cheatsheet
/// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
pub(crate) fn generate_hash_pbkdf2(data: String) -> Result<String, crate::Error> {
    let hasher = Pbkdf2::new(
        pbkdf2::Algorithm::Pbkdf2Sha512,
        Params::new(210_000).unwrap(),
    );

    // Salt
    let salt = SaltString::generate();
    // Hash
    match hasher.hash_password_with_salt(data.as_bytes(), &salt.to_salt()) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => Err(crate::Error::HashingError(format!(
            "Error hashing password: {}",
            e
        ))),
    }
}

pub(crate) fn verify_hash_pbkdf2(data: String, hash: String) -> Result<bool, crate::Error> {
    let hasher = Pbkdf2::new(
        pbkdf2::Algorithm::Pbkdf2Sha512,
        Params::new(210_000).unwrap(),
    );

    let parsed_hash = match PasswordHash::new(&hash) {
        Ok(h) => h,
        Err(e) => {
            return Err(crate::Error::HashingError(format!(
                "Error parsing password hash: {}",
                e
            )));
        }
    };

    match hasher.verify_password(data.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::crypto::hashing::orm_pbkdf2::{generate_hash_pbkdf2, verify_hash_pbkdf2};

    #[test]
    fn test_hasing_pbkdf2() {
        let clear_text = "super-secret-geekorm-password";
        let cipher_text = generate_hash_pbkdf2(clear_text.to_string()).unwrap();

        // Split out the PHC and make sure it follows the spec
        let comps: Vec<&str> = cipher_text.split('$').collect();
        assert_eq!(comps.len(), 5);
        assert_eq!(comps.get(1).unwrap(), &"pbkdf2-sha512");

        let params: Vec<&str> = comps.get(2).unwrap().split(',').collect();
        for param in params.iter() {
            if param.starts_with("i=") {
                assert_eq!(param, &"i=210000");
            } else if param.starts_with("l=") {
                assert_eq!(param, &"l=32");
            }
        }

        let salt = comps.get(2).unwrap();
        assert!(salt.len() > 12);

        let result = verify_hash_pbkdf2(clear_text.to_string(), cipher_text).unwrap();
        assert_eq!(result, true);
    }
}
