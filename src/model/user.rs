use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::service::authentication::hash_password;
use uuid;
use uuid::Uuid;

// #[derive(Hash, Eq, Serialize, Deserialize, Clone, Debug, Display, PartialEq)]
// #[repr(transparent)]
// pub struct UserID(Uuid);
pub(crate) type UserID = Uuid;
pub(crate) type Nonce = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConfirmationStatus {
    PENDING,
    CONFIRMED,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub(crate) user_id: UserID,
    pub(crate) username: String,
    pub(crate) password_hash: String,
    pub(crate) email_state: ConfirmationStatus,
    pub(crate) confirmation_nonce: Option<Nonce>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        let password_hash = hash_password(&password);
        Self {
            user_id: UserID::new_v4(),
            username,
            password_hash,
            email_state: ConfirmationStatus::PENDING,
            confirmation_nonce: None,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}, uuid={}", self.username, self.user_id)
    }
}

impl User {
    pub fn example() -> Self {
        Self::new("Peter".to_string(), "asdfasdfasdf".to_string())
    }
}
