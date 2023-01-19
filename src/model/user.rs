use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type UserID = i64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub(crate) username: String,
    pub(crate) password_hash: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}", self.username)
    }
}
