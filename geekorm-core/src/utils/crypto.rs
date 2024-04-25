/// This module contains functions for generating random strings

#[cfg(feature = "rand")]
use rand::Rng;

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
