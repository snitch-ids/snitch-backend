pub mod mongodb;
pub mod redis;
pub mod user;

use anyhow::Result;
use async_trait::async_trait;

use crate::model::MessageBackend;

use self::user::User;

#[async_trait]
pub trait Persist {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()>;

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>>;

    async fn add_user(&mut self, user: &User) -> Result<()>;
    async fn get_users(&mut self) -> Result<Vec<User>>;
    async fn get_user_by_email(&mut self, email: &str) -> Result<User>;
}
