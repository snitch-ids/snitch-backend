use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type UserID = i64;

#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct User {
    pub(crate) username: String,
    pub(crate) password_hash: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}", self.username)
    }
}

impl User {
    pub fn example () -> Self {
        Self{ username: "Peter".to_string(), password_hash: "asdfasdfasdf".to_string() }
    }
}