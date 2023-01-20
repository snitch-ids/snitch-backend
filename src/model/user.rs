
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use uuid;
use uuid::{Uuid};

// #[derive(Hash, Eq, Serialize, Deserialize, Clone, Debug, Display, PartialEq)]
// #[repr(transparent)]
// pub struct UserID(Uuid);
pub(crate) type UserID = Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub(crate) user_id: UserID,
    pub(crate) username: String,
    pub(crate) password_hash: String,
}

impl User {
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            user_id: UserID::new_v4(),
            username,
            password_hash,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}, uuid={}", self.username, self.user_id)
    }
}
