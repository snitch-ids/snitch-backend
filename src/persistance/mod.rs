pub mod mongodb;
pub mod redis;
pub mod users;

use anyhow::Result;
use async_trait::async_trait;

use crate::model::MessageBackend;

#[async_trait]
pub trait Persist {
    async fn add_message(&mut self, message: MessageBackend) -> Result<()>;

    async fn find_messages(&mut self, hostname: String) -> Result<Vec<MessageBackend>>;
}
