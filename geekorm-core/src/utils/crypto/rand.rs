use rand::Rng;

/// Character set for generating random strings
pub const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

/// Generate a random string of a given length
pub fn generate_random_string(length: usize, prefix: impl Into<String>) -> String {
    let mut rng = rand::rng();
    let mut random_string = String::new();
    // Generate a random string of the given length using uppercase, lowercase and numbers
    for _ in 0..length {
        let random_char = CHARSET[rng.random_range(0..CHARSET.len())] as char;
        random_string.push(random_char);
    }
    prefix.into() + &random_string
}
