pub mod mongodb;
pub mod redis;
pub mod users;

use crate::model::message::MessageBackend;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Persist {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()>;

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>>;
}
