use argonautica::{Error, Hasher, Verifier};

pub fn hash_password(password: &str) -> String {
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(password)
        .with_secret_key(
            "\
            secret key that you should really store in a .env file \
            instead of in code, but this is just an example\
        ",
        )
        .hash()
        .unwrap();
    hash
}

pub fn valid_hash(hash: &str, password: &str) -> Result<bool, Error> {
    let mut verifier = Verifier::default();
    verifier
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(
            "\
            secret key that you should really store in a .env file \
            instead of in code, but this is just an example\
        ",
        )
        .verify()
}

#[test]
pub fn test_hasher_valid() {
    let password = "password";
    let hash = hash_password(password);
    assert!(valid_hash(&hash, password).unwrap());
}

#[test]
pub fn test_hasher_invalid() {
    let password = "password";
    let hash = hash_password(password);
    assert!(valid_hash(&hash, "password_false").unwrap());
}

#[test]
fn test_example_password() {
    let password = "grr";
    let hash = hash_password(password);
    println!("{hash}");
}
