use crate::errors::SnitchError;


use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

type Result<T> = std::result::Result<T, SnitchError>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct User {
    id: i64,
    name: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id={} | name={}", self.id, self.name)
    }
}

pub struct Users {
    users: HashMap<i64, User>,
}

impl Users {
    pub fn add_user(&mut self, user: User) -> Result<User> {
        self.users.insert(user.id, user.clone());
        Ok(user)
    }

    pub fn delete_user(&mut self, user_id: i64) -> Result<User> {
        let user = self.users.remove(&user_id).expect("failed deleting user");
        Ok(user)
    }

    pub fn get_users(&self) -> Result<Vec<User>> {
        let users = self.users.values().cloned().collect();
        Ok(users)
    }

    pub fn get_user_by_id(&self, user_id: i64) -> Result<User> {
        let user = self.users.get(&user_id);
        let user = match user {
            Some(user) => user,
            None => return Err(SnitchError {}),
        };
        Ok(user.clone())
    }

    pub fn example() -> Self {
        let test_user = User {
            id: 1,
            name: "testuser".to_string(),
        };
        let mut users = Users {
            users: Default::default(),
        };
        users
            .add_user(test_user)
            .expect("Failed setting up example");
        users
    }
}
