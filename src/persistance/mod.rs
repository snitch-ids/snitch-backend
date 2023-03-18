pub mod redis;
pub mod token;
pub mod users;

use crate::model::message::MessageBackend;
use crate::model::user::{UserID};

use anyhow::Result;
use async_trait::async_trait;


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
        format!("messages:{}:{}:", self.user_id, self.hostname)
    }
}

#[async_trait]
pub trait PersistMessage {
    async fn add_message(
        &mut self,
        message_key: &MessageKey,
        message: &MessageBackend,
    ) -> Result<()>;

    async fn find_messages(&mut self, message_key: &MessageKey) -> Result<Vec<MessageBackend>>;
}
