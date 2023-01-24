pub mod mongodb;
pub mod redis;
pub mod token;
pub mod users;

use crate::model::message::MessageBackend;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PersistMessage {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()>;

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>>;
}
