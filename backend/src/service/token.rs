use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn random_alphanumeric_string(length: u32) -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    token
}
