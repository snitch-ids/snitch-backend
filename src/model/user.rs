use derive_more::{Display, FromStr};

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::api::registration::RegistrationRequest;
use crate::service::authentication::hash_password;

use uuid;
use uuid::Uuid;

// #[derive(Hash, Eq, Serialize, Deserialize, Clone, Debug, Display, PartialEq)]
// pub struct UserID(Uuid);

pub(crate) type Nonce = String;

#[derive(
    Serialize, Deserialize, Debug, Display, FromStr, Hash, Ord, Eq, PartialOrd, PartialEq, Clone,
)]
pub struct UserID(String);

impl UserID {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl From<String> for UserID {
    fn from(value: String) -> Self {
        UserID(value)
    }
}

// impl FromRedisValue for UserID {
//     fn from_redis_value(v: &Value) -> RedisResult<Self> {
//         return match v {
//             Value::Nil => {UserID("".to_string())},
//             Value::Data(d) => {},
//             _ => println!("fail")
//         }
//     }
//
//     fn from_redis_values(items: &[Value]) -> RedisResult<Vec<Self>> {
//         Ok(vec![UserID::new_v4()])
//     }
//
//     fn from_byte_vec(_vec: &[u8]) -> Option<Vec<Self>> {
//         None
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub user_id: UserID,
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        let password_hash = hash_password(&password);
        Self {
            user_id: UserID::new(),
            username,
            password_hash,
        }
    }

    #[allow(dead_code)]
    pub fn example() -> Self {
        Self::new("Peter".to_string(), "asdfasdfasdf".to_string())
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "name={}, uuid={}", self.username, self.user_id)
    }
}

impl From<RegistrationRequest> for User {
    fn from(value: RegistrationRequest) -> Self {
        User::new(value.username, value.password)
    }
}

// impl FromRedisValue for User{
//     fn from_redis_value(v: &Value) -> RedisResult<Self> {
//         match v {
//             Value::Nil => {println!("nil")}
//             Value::Int(i) => {println!(" int {}", i)}
//             Value::Data(d) => {println!("data {:?}", d)}
//             Value::Bulk(d) => {println!("biuld {:?}", d)}
//             Value::Status(d) => {println!("status {:?}", d)}
//             Value::Okay => {}
//         }
//
//         Ok(User::example())
//     }
// }
