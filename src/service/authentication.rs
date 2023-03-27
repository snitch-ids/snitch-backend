use argonautica::{Hasher, Verifier};
use log::error;

use lazy_static::lazy_static;

lazy_static! {
    static ref SECRET_KEY: String =
        std::env::var("SNITCH_PASSWORD_SECRET").expect("SNITCH_PASSWORD_SECRET not defined");
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(password)
        .with_secret_key(&*SECRET_KEY)
        .hash()
        .unwrap();
    hash
}

pub fn valid_hash(hash: &str, password: &str) -> bool {
    let mut verifier = Verifier::default();
    match verifier
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(&*SECRET_KEY)
        .verify()
    {
        Ok(result) => return result,
        Err(e) => {
            error!("{}", e);
        }
    }
    false
}

#[test]
pub fn test_hasher_valid() {
    let password = "password";
    let hash = hash_password(password);
    assert!(valid_hash(&hash, password));
}

#[test]
pub fn test_hasher_invalid() {
    let password = "password";
    let hash = hash_password(password);
    assert!(!valid_hash(&hash, "password_false"));
}
