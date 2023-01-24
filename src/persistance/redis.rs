use crate::errors::ServiceError;
use crate::model::message::MessageBackend;
use crate::model::user::{Nonce, User, UserID};
use crate::persistance::PersistMessage;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use log::info;
use redis::AsyncCommands;
use redis::Commands;
use redis::JsonAsyncCommands;
use redis::{aio, RedisResult};
use serde::Serialize;
use serde_json::json;
use std::fmt::Error;

pub struct RedisDatabaseService {
    pub client: redis::Client,
    pub connection: aio::Connection,
}

#[derive(Serialize)]
struct EMPTY {}

impl RedisDatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let connection = client.get_async_connection().await?;
        let mut db = RedisDatabaseService { client, connection };
        db.setup().await;
        Ok(db)
    }

    async fn setup(&mut self) {
        let _: () = self
            .connection
            .json_set("user", ".", &json!(EMPTY {}))
            .await
            .unwrap();
        let _: () = self
            .connection
            .json_set("user_pending", ".", &json!(EMPTY {}))
            .await
            .unwrap();
    }

    pub async fn add_user(&mut self, user: &User) {
        let user_id = user.user_id;
        let _: () = self
            .connection
            .json_set("user", user_id.to_string(), &json!(user))
            .await
            .unwrap();
    }

    pub async fn add_user_pending(&mut self, user: &User, nonce: &Nonce) {
        let _: () = self
            .connection
            .json_set("user_pending", nonce, &json!(user))
            .await
            .unwrap();
    }

    pub async fn confirm_user_pending(&mut self, nonce: &Nonce) -> Result<()> {
        let user = self.get_user_pending(&nonce).await;
        self.add_user(&user).await;
        self.delete_user_pending(&nonce).await;
        info!("confirmed {}", user);
        Ok(())
    }

    pub async fn get_user_pending(&mut self, nonce: &Nonce) -> User {
        let user_str: String = self
            .connection
            .json_get("user_pending", nonce)
            .await
            .unwrap();
        serde_json::from_str(&user_str).unwrap()
    }

    pub async fn delete_user_pending(&mut self, nonce: &Nonce) {
        let _: () = self
            .connection
            .json_del("user_pending", nonce)
            .await
            .unwrap();
    }

    pub async fn get_user_by_id(mut self, user_id: &UserID) -> User {
        let user_str: String = self
            .connection
            .json_get("user", user_id.to_string())
            .await
            .unwrap();
        serde_json::from_str(&user_str).unwrap()
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        todo!()
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
    let test_user = User::example();
    let mut db = RedisDatabaseService::new("redis://127.0.0.1:6379")
        .await
        .unwrap();

    let x = db.add_user(&test_user).await;
    let x = db.get_user_by_id(&test_user.user_id).await;
}
