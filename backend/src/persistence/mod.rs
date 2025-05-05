pub mod redis;
pub mod token;

use crate::model::message::MessageBackend;
use crate::model::user::UserID;
use std::format;

use anyhow::Result;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MessageKey {
    pub user_id: UserID,
    pub hostname: String,
}

impl Default for MessageKey {
    fn default() -> Self {
        MessageKey {
            user_id: UserID::default(),
            hostname: "default_hostname".to_string(),
        }
    }
}

impl MessageKey {
    fn to_redis_key(&self) -> String {
        format!("messages:{}:{}", self.user_id, self.hostname)
    }
}

pub trait PersistMessage {
    async fn add_message(
        &mut self,
        message_key: &MessageKey,
        message: &MessageBackend,
    ) -> Result<()>;

    async fn find_messages(&mut self, message_key: &MessageKey) -> Result<Vec<MessageBackend>>;
    async fn get_hostnames_of_user(&mut self, user_id: &UserID) -> Result<Vec<String>>;
}
