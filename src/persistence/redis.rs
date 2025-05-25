use crate::errors::{APIError, APIInternalError};
use crate::model::message::MessageBackend;
use crate::model::user::{Nonce, User, UserID};
use crate::persistence::{MessageKey, PersistMessage};
use anyhow::{Error as AnyhowError, Ok, Result as AnyhowResult};
use chatterbox::dispatcher::email::Email;
use chatterbox::dispatcher::slack::Slack;
use chatterbox::dispatcher::telegram::Telegram;
use chatterbox::dispatcher::Sender;
use log::{debug, info};
use redis::JsonAsyncCommands;
use redis::{aio, RedisResult};
use redis::{AsyncCommands, FromRedisValue};
use serde::{Deserialize, Serialize};
use serde_json::error::Error as SerdeJsonError;
use serde_json::json;
use std::env;
use std::result::Result::Ok as StdOk;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum PersistenceError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serde error: {0}")]
    Serde(#[from] SerdeJsonError),
    #[error("No user: {0}")]
    NoUser(String),
}

#[derive(Debug)]
pub struct RedisDatabaseService {
    pub connection: aio::MultiplexedConnection,
}

const MAX_MESSAGES: isize = 1000;
const MINUTE: usize = 60;
const DAY: usize = 60 * MINUTE * 24;

enum TTL {
    PendingUser = (15 * MINUTE) as isize,
    Message = DAY as isize,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub(crate) struct NotificationSettings {
    telegram: Option<Telegram>,
    slack: Option<Slack>,
    email: Option<Email>,
}

impl FromRedisValue for NotificationSettings {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        let s: String = FromRedisValue::from_redis_value(v)?;
        StdOk(serde_json::from_str(&s)?)
    }
}

impl From<NotificationSettings> for Sender {
    fn from(value: NotificationSettings) -> Self {
        Self {
            telegram: value.telegram,
            email: value.email,
            slack: value.slack,
        }
    }
}

impl RedisDatabaseService {
    pub async fn new() -> AnyhowResult<Self> {
        let url = Self::get_redis_url();
        debug!("connecting to {url}");
        let client = redis::Client::open(url)?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(RedisDatabaseService { connection })
    }

    pub(crate) fn get_redis_url() -> String {
        let url = env::var("SNITCH_REDIS_URL").expect("SNITCH_REDIS_URL not defined");
        info!("connecting to redis {}", url);

        let password =
            env::var("SNITCH_REDIS_PASSWORD").expect("SNITCH_REDIS_PASSWORD not defined");
        let url = format!("redis://:{}@{}", password, url);
        url
    }

    pub async fn add_user_index(&mut self, user: &User) {
        let user_id = &user.user_id;
        let email = &user.email;
        let _: () = self
            .connection
            .set(format!("user_email:{email}"), user_id.to_string())
            .await
            .unwrap();
    }

    pub async fn add_user(&mut self, user: User) -> Result<User, PersistenceError> {
        let user_id = &user.user_id;
        if let StdOk(already_existing_user) = self.get_user_by_email(&user.email).await {
            info!("not adding user as user already exists: {user}");
            return StdOk(already_existing_user);
        }
        let _: () = self
            .connection
            .json_set(format!("user:{user_id}"), "$", &json!(user))
            .await?;
        self.add_user_index(&user).await;
        StdOk(user)
    }

    pub async fn delete_user(&mut self, user_id: &UserID) {
        let _: () = self
            .connection
            .json_del(format!("user:{user_id}"), ".")
            .await
            .unwrap();
    }

    pub async fn add_user_pending(&mut self, user: &User, nonce: &Nonce) -> AnyhowResult<()> {
        if self.get_user_by_email(&user.email).await.is_ok() {
            info!("not adding user pending as user already exists: {user}");
            return Err(AnyhowError::new(APIInternalError::UserAlreadyExists(
                user.clone(),
            )));
        }
        let key = format!("user_pending:{nonce}");
        self.connection.json_set(&key, "$", &json!(user)).await?;
        self.connection
            .expire(&key, TTL::PendingUser as i64)
            .await?;
        Ok(())
    }

    pub async fn confirm_user_pending(&mut self, nonce: &Nonce) -> AnyhowResult<()> {
        let user = self.get_user_pending(nonce).await?;
        self.add_user(user).await?;
        self.delete_user_pending(nonce).await;
        Ok(())
    }

    pub async fn get_user_pending(&mut self, nonce: &Nonce) -> AnyhowResult<User> {
        info!("get pending user. nonce: {nonce}");
        let user_str: String = self
            .connection
            .json_get(format!("user_pending:{nonce}"), ".")
            .await?;
        Ok(serde_json::from_str(&user_str)?)
    }

    pub async fn delete_user_pending(&mut self, nonce: &Nonce) {
        let _: () = self
            .connection
            .json_del(format!("user_pending:{nonce}"), ".")
            .await
            .unwrap();
    }

    pub async fn get_user_by_id(&mut self, user_id: &UserID) -> Result<User, PersistenceError> {
        info!("get user by user_id {user_id}");
        let user_str: String = self
            .connection
            .json_get(format!("user:{user_id}"), ".")
            .await?;
        let user = serde_json::from_str(&user_str)?;
        StdOk(user)
    }

    pub async fn get_user_by_email(&mut self, email: &str) -> Result<User, PersistenceError> {
        info!("get user by email {email}");
        if let Some(result) = self.connection.get(format!("user_email:{email}")).await? {
            info!("found user?: {:?}", result);
            let user_id = String::from_redis_value(&result)?;
            return self.get_user_by_id(&user_id.into()).await;
        };
        Err(PersistenceError::NoUser(email.to_string()))
    }

    pub(crate) async fn get_notification_settings(
        &mut self,
        user_id: &UserID,
    ) -> NotificationSettings {
        self.connection
            .json_get(format!("notification_settings:{user_id}"), ".")
            .await
            .unwrap_or_default()
    }

    pub(crate) async fn set_notification_settings(
        &mut self,
        user_id: &UserID,
        notification_settings: NotificationSettings,
    ) {
        self.connection
            .json_set(
                format!("notification_settings:{user_id}"),
                ".",
                &notification_settings,
            )
            .await
            .unwrap()
    }
}

#[allow(dead_code)]
#[cfg(debug_assertions)]
fn load_demo_notification_settings() -> NotificationSettings {
    let slack = match std::env::var("CHATTERBOX_SLACK_WEBHOOK_URL") {
        StdOk(webhook_url) => {
            info!("Using Slack dispatcher");
            let channel = std::env::var("CHATTERBOX_SLACK_CHANNEL")
                .expect("CHATTERBOX_SLACK_CHANNEL not defined");
            Some(Slack {
                webhook_url,
                channel,
            })
        }
        Err(_) => {
            info!("CHATTERBOX_SLACK_WEBHOOK_URL not defined");
            None
        }
    };
    let telegram = match std::env::var("CHATTERBOX_TELEGRAM_BOT_TOKEN") {
        StdOk(bot_token) => {
            info!("Using Telegram dispatcher");
            let chat_id = std::env::var("CHATTERBOX_TELEGRAM_CHAT_ID")
                .expect("CHATTERBOX_TELEGRAM_CHAT_ID not defined");
            Some(Telegram { bot_token, chat_id })
        }
        Err(_) => {
            info!("CHATTERBOX_TELEGRAM_BOT_TOKEN not defined");
            None
        }
    };

    NotificationSettings {
        telegram,
        slack,
        email: None,
    }
}

impl PersistMessage for RedisDatabaseService {
    async fn add_message(
        &mut self,
        key: &MessageKey,
        message: &MessageBackend,
    ) -> AnyhowResult<()> {
        let key = key.to_redis_key();
        let _: () = self.connection.rpush(&key, message).await?;
        info!("storing in database: {:?}... finished", message);
        self.connection.expire(&key, TTL::Message as i64).await?;

        Ok(())
    }

    async fn find_messages(&mut self, key: &MessageKey) -> AnyhowResult<Vec<MessageBackend>> {
        let responses: Vec<String> = self
            .connection
            .lrange(key.to_redis_key(), 0, MAX_MESSAGES)
            .await?;
        let messages = responses
            .iter()
            .map(|response| serde_json::from_str(response).unwrap())
            .collect();
        Ok(messages)
    }

    async fn get_hostnames_of_user(&mut self, user_id: &UserID) -> AnyhowResult<Vec<String>> {
        let key = format!("messages:{user_id}:*");
        let keys: Vec<String> = self.connection.keys(key).await?;
        let hostnames = keys
            .iter()
            .map(|item| item.split(':').last().unwrap().to_string())
            .collect::<Vec<String>>();
        Ok(hostnames)
    }
}

#[tokio::test]
async fn test_add_delete_user() {
    use crate::model::user::User;
    let mut test_user = User::example();
    test_user.email = "x.x@x.x".to_string();
    let mut db = RedisDatabaseService::new().await.unwrap();

    let test_user = db.add_user(test_user).await.unwrap();
    let _x = db.get_user_by_id(&test_user.user_id).await;
    let x = db.get_user_by_email(&test_user.email).await.unwrap();
    assert_eq!(x.email, test_user.email);
    assert_eq!(x.user_id, test_user.user_id);

    db.delete_user(&test_user.user_id).await;
    // Test this to improve error handling
    // assert_eq!(db.get_user_by_name(&test_user.email).await.ok(), None);
}

#[tokio::test]
async fn test_add_messages() {
    use crate::model::user::User;
    let mut test_user = User::example();
    test_user.email = "x.x@x.x".to_string();
    let mut db = RedisDatabaseService::new().await.unwrap();
    let mut test_message = MessageBackend::default();
    let test_user = db.add_user(test_user).await.unwrap();

    let n_hostnames = 3;
    for i in 0..n_hostnames {
        test_message.hostname = format!("testhostname-{}", i);
        let key = MessageKey {
            user_id: test_user.user_id.clone(),
            hostname: test_message.hostname.clone(),
        };
        db.add_message(&key, &test_message).await.unwrap();
        assert_eq!(db.find_messages(&key).await.unwrap().len(), 1);
    }

    let hostnames = db.get_hostnames_of_user(&test_user.user_id).await.unwrap();
    assert_eq!(hostnames.len(), n_hostnames);

    db.delete_user(&test_user.user_id).await;
}
