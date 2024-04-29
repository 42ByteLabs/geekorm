/// Cryptography module
pub mod crypto;

#[cfg(feature = "rand")]
pub use crypto::generate_random_string;

#[cfg(feature = "hash")]
pub use crypto::generate_hash;
