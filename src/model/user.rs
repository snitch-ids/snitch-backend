use actix_jwt_auth_middleware::FromRequest;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct User {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}", self.username)
    }
}
