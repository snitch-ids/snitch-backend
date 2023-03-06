use crate::model::message::MessageBackend;
use crate::model::user::{Nonce, User, UserID};
use crate::persistance::PersistMessage;
use anyhow::{Ok, Result};
use async_trait::async_trait;

use log::{debug, info};
use redis::aio;
use redis::JsonAsyncCommands;
use redis::{AsyncCommands, FromRedisValue, Value};
use serde::Serialize;
use serde_json::json;

pub struct RedisDatabaseService {
    pub client: redis::Client,
    pub connection: aio::Connection,
}

#[derive(Serialize)]
struct Empty {}

impl RedisDatabaseService {
    pub async fn new() -> Result<Self> {
        let url = std::env::var("SNITCH_REDIS_URL").expect("SNITCH_REDIS_URL not defined");
        info!("connecting to redis {}", url);

        let password =
            std::env::var("SNITCH_REDIS_PASSWORD").expect("SNITCH_REDIS_PASSWORD not defined");
        let url = format!("redis://:{}@{}", password, url);
        debug!("connecting to {url}");
        let client = redis::Client::open(url)?;
        let connection = client.get_async_connection().await?;
        Ok(RedisDatabaseService { client, connection })
    }

    pub async fn add_user_index(&mut self, user: &User) {
        let user_id = &user.user_id;
        let username = &user.username;
        let _: () = self
            .connection
            .set(format!("user_usernames:{username}"), user_id.to_string())
            .await
            .unwrap();
    }

    pub async fn add_user(&mut self, user: &User) {
        let user_id = &user.user_id;
        let _: () = self
            .connection
            .json_set(format!("user:{user_id}"), "$", &json!(user))
            .await
            .unwrap();
        self.add_user_index(user).await;
    }

    pub async fn add_user_pending(&mut self, user: &User, nonce: &Nonce) {
        let _: () = self
            .connection
            .json_set(format!("user_pending:{nonce}"), "$", &json!(user))
            .await
            .unwrap();
    }

    pub async fn confirm_user_pending(&mut self, nonce: &Nonce) -> Result<()> {
        let user = self.get_user_pending(nonce).await;
        self.add_user(&user).await;
        self.delete_user_pending(nonce).await;
        info!("confirmed {}", user);
        Ok(())
    }

    pub async fn get_user_pending(&mut self, nonce: &Nonce) -> User {
        let user_str: String = self
            .connection
            .json_get(format!("user_pending:{nonce}"), ".")
            .await
            .unwrap();
        serde_json::from_str(&user_str).unwrap() // impl FromStr
    }

    pub async fn delete_user_pending(&mut self, nonce: &Nonce) {
        let _: () = self
            .connection
            .json_del(format!("user_pending:{nonce}"), ".")
            .await
            .unwrap();
    }

    pub async fn get_user_by_id(&mut self, user_id: &UserID) -> User {
        info!("user_id {user_id}");
        let user_str: String = self
            .connection
            .json_get(format!("user:{user_id}"), ".")
            .await
            .unwrap();
        serde_json::from_str(&user_str).unwrap()
    }

    pub async fn get_user_by_name(&mut self, username: &str) -> Result<User> {
        info!("get user by name {username}");
        let result: Value = self
            .connection
            .get(format!("user_usernames:{username}"))
            .await?;
        let user_id: UserID = String::from_redis_value(&result).unwrap_or_default().into();
        Ok(self.get_user_by_id(&user_id).await)
    }

    pub async fn _get_user_by_name_index(&mut self, _username: &str) {
        // Create index for username
        // FT.CREATE idx:username
        //   ON JSON
        //   PREFIX 1 "user:"
        //   SCHEMA $.username AS username TEXT
        //
        // Then search with
        // FT.SEARCH idx:username_pending "1"
        // redis returned list of key-value pairs
        // key: user:b545cc19-169a-425f-8f97-3cff9d6237fc
        // value: {\"user_id\":\"b545cc19-169a-425f-8f97-3cff9d6237fc\",\"username\":\"1\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$n9HwKN4bu7cUCojo08Tx8ke9Lr0gUBSqQrfE7h67oKE$LDeeOCtslWxiCEQdxx4xAUFsyzczlhC+FX1C/rwcoqk\"}
        // key: user:b545cc19-169a-425f-8f97-3cff9d6237fc
        // value: {\"user_id\":\"b545cc19-169a-425f-8f97-3cff9d6237fc\",\"username\":\"1\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$n9HwKN4bu7cUCojo08Tx8ke9Lr0gUBSqQrfE7h67oKE$LDeeOCtslWxiCEQdxx4xAUFsyzczlhC+FX1C/rwcoqk\"}//
        // ....

        // Failed to parse:
        // bulk(int(4), string-data('"user:9511c06b-4f59-4fca-8ac5-fa544d7c1cdf"'), bulk(string-data('"$"'), string-data('"{\"user_id\":\"9511c06b-4f59-4fca-8ac5-fa544d7c1cdf\",\"username\":\"Peter\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$fpL+GDtUBZ1MMzgppZ3VUaz11w+rBqTr3umNY1kDxWw$TZLCDdX4ZFAMykPWodwnwdDJw6lS/QyG9SaGK31quSI\"}"')), string-data('"user:09f53c2e-421a-4b88-9aa4-5084e4c3111f"'), bulk(string-data('"$"'), string-data('"{\"user_id\":\"09f53c2e-421a-4b88-9aa4-5084e4c3111f\",\"username\":\"Peter\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$02yKNoQzWk1NKw5t7iHh0EpEQqWOPfK9U6h42D3lWcI$E0dp+o5tuJqzMDZ7v8F+nOB0sSEL7l+RGUOzHS1MRw8\"}"')), string-data('"user:9fd64f6f-bb5c-4a99-868a-5280191d1880"'), bulk(string-data('"$"'), string-data('"{\"user_id\":\"9fd64f6f-bb5c-4a99-868a-5280191d1880\",\"username\":\"Peter\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$9zKdgiZZzL0P9Kp5dTq4MgtqTG5RN4q7SVAw3dlN5yc$fGNscohzAQlpjEim6KXx0yrBntGGTl1HfJGnR1+sUdM\"}"')), string-data('"user:35c23941-dc22-4721-b574-7ea31a887cc9"'), bulk(string-data('"$"'), string-data('"{\"user_id\":\"35c23941-dc22-4721-b574-7ea31a887cc9\",\"username\":\"Peter\",\"password_hash\":\"$argon2id$v=19$m=4096,t=192,p=8$smWqFAA9lIDGWFYroGHlwdMxWJaBfCxQzOYCYAvhjYk$BQlWXTQnk8XfQfRbQq8Ll93W067elMWWvE+M5JUMrSM\"}"'))))
        //{  N returned objects,     user-id                                     } , {   Path,                 JSON-as-string-data ...
        //                           user-id                                     } , {   Path,                 JSON-as-string-data ...
        //                           user-id                                     } , {   Path,                 JSON-as-string-data ...
        //                           user-id                                     } , {   Path,                 JSON-as-string-data ...
        // let result = redis::cmd("FT.SEARCH").arg("idx:username").arg(username).query_async(&mut self.connection).await.unwrap();
        //
        // match resulta {
        //     Value::Nil => {}
        //     Value::Int(_) => {}
        //     Value::Data(_) => {}
        //     Value::Bulk(v) => {User::from_redis_values(&*v).expect("TODO: panic message");}
        //     Value::Status(_) => {}
        //     Value::Okay => {}
        // }
        // for (_, v) in result.as_sequence()
        //         .unwrap()
        //         .iter()
        //         .skip(1)
        //         .into_iter()
        //         .tuples()
        // {
        //     println!("{v:?}");
        //     User::from_redis_value(v).expect("TODO: panic message");
        // }
        // result.to_vec();
    }
}

#[async_trait]
impl PersistMessage for RedisDatabaseService {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()> {
        info!("storing in database: {:?}", message);
        let _: () = self
            .connection
            .rpush(&message.hostname, message)
            .await
            .unwrap();
        info!("storing in database: {:?}... finished", message);

        Ok(())
    }

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>> {
        let responses: Vec<String> = self.connection.lrange(hostname, 0, -2).await?;
        let messages = responses
            .iter()
            .map(|response| serde_json::from_str(response).unwrap())
            .collect();
        Ok(messages)
    }
}

#[tokio::test]
async fn test_add_user() {
    use crate::model::user::User;
    let mut test_user = User::example();
    test_user.username = "xxxx".to_string();
    let mut db = RedisDatabaseService::new().await.unwrap();

    db.add_user(&test_user).await;
    let _x = db.get_user_by_id(&test_user.user_id).await;
    let x = db.get_user_by_name(&test_user.username).await.unwrap();
    assert_eq!(x.username, test_user.username);
    assert_eq!(x.user_id, test_user.user_id);
}
