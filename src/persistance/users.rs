use crate::errors::SnitchError;

use actix_jwt_auth_middleware::{
    AuthResult, AuthenticationService, Authority, CookieSigner, FromRequest,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) password: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id={} | name={}", self.id, self.name)
    }
}

pub struct Users {
    users: HashMap<i64, User>,
}

impl Users {
    pub fn add_user(&mut self, user: User) -> Result<User, Box<dyn std::error::Error>> {
        self.users.insert(user.id, user.clone());
        Ok(user)
    }

    pub fn delete_user(&mut self, user_id: i64) -> Result<User, Box<dyn std::error::Error>> {
        let user = self.users.remove(&user_id).expect("failed deleting user");
        Ok(user)
    }

    pub fn get_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let users = self.users.values().cloned().collect();
        Ok(users)
    }

    pub fn get_user_by_id(&self, user_id: i64) -> Result<User, Box<dyn std::error::Error>> {
        let user = self.users.get(&user_id);
        let user = match user {
            Some(user) => user,
            None => return Err(Box::try_from(Error).unwrap()),
        };
        Ok(user.clone())
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        let users = self
            .users
            .iter()
            .map(|(_, user)| user)
            .filter(|user| user.name == username)
            .collect::<Vec<&User>>();
        return if users.len() != 1 {
            None
        } else {
            Some(users[0])
        };
    }

    pub fn example() -> Self {
        let test_user = User {
            id: 1,
            name: "testuser".to_string(),
            password: "grr".to_string(),
        };
        let mut users = Users {
            users: Default::default(),
        };
        users
            .add_user(test_user)
            .expect("Failed setting up example");
        users
    }

    pub fn valid_password(&self, username: &str, password: &str) -> bool {
        return match self.get_user_by_name(username) {
            Some(user) => user.password == password,
            _ => false,
        };
    }
}

#[test]
fn test_valid_password() {
    let users = Users::example();
    assert_eq!(users.valid_password("noneexistend", "password"), false);
    assert_eq!(users.valid_password("testuser", "password"), false);
    assert_eq!(users.valid_password("testuser", "grr"), true);
}
