use crate::model::message::MessageToken;
use crate::model::user::UserID;

use crate::errors::APIError;
use crate::service::token::random_alphanumeric_string;
use log::{error, info};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::str::FromStr;
use tokio::sync::Mutex;

const TOKEN_LENGTH: u32 = 32;

pub struct TokenStore {
    pub connection: MultiplexedConnection,
}

impl TokenStore {
    pub async fn create_token_for_user_id(&mut self, user_id: &UserID) -> MessageToken {
        info!("create token for user_id {}", user_id);
        let token = random_alphanumeric_string(TOKEN_LENGTH);
        let key_user_id_to_token = format!("user_id_to_token:{user_id}");
        let _: u8 = self
            .connection
            .sadd(&key_user_id_to_token, &token)
            .await
            .map_err(|e| error!("{}", e))
            .unwrap();

        let key_token_to_user_id: String = format!("token_to_user_id:{token}");
        let _: u8 = self
            .connection
            .hset(key_token_to_user_id, "user_id", user_id.to_string())
            .await
            .expect("failed adding token");
        token
    }

    pub async fn get_token_of_user_id(&mut self, user_id: &UserID) -> Option<Vec<MessageToken>> {
        let key = format!("user_id_to_token:{user_id}");
        self.connection.smembers(&key).await.ok()
    }

    pub async fn get_user_id_of_token(&mut self, token: &MessageToken) -> Option<UserID> {
        let key_token_to_user_id = format!("token_to_user_id:{token}");
        let result: String = self
            .connection
            .hget(key_token_to_user_id, "user_id")
            .await
            .inspect_err(|e| error!("{e}"))
            .ok()?;
        UserID::from_str(&result).ok()
    }

    pub async fn delete_token(&mut self, token: &MessageToken) -> Result<(), APIError> {
        let user_id = self
            .get_user_id_of_token(token)
            .await
            .ok_or(APIError::InternalServerError)?;
        let key_token_to_user_id = format!("token_to_user_id:{token}");
        let key_user_id_to_token = format!("user_id_to_token:{user_id}");

        self.connection
            .del(key_token_to_user_id)
            .await
            .inspect_err(|e| error!("{e}"))
            .map_err(|_| APIError::InternalServerError)?;
        self.connection
            .srem(key_user_id_to_token, token)
            .await
            .inspect_err(|e| error!("{e}"))
            .map_err(|_| APIError::InternalServerError)?;
        Ok(())
    }
}

pub struct TokenState {
    pub token: Mutex<TokenStore>,
}

impl TokenState {
    pub fn new(connection: MultiplexedConnection) -> TokenState {
        Self {
            token: Mutex::new(TokenStore { connection }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::redis::RedisDatabaseService;

    #[tokio::test]
    async fn test_token_store() {
        let db = RedisDatabaseService::new().await.unwrap();
        let mut store = TokenStore {
            connection: db.connection,
        };
        let user_id = UserID::new();
        let _ = store.create_token_for_user_id(&user_id).await;
        let _ = store.create_token_for_user_id(&user_id).await;
        let token_list = store.get_token_of_user_id(&user_id).await.unwrap();
        assert_eq!(token_list.len(), 2);
    }

    #[tokio::test]
    async fn test_user_id_of_token() {
        let db = RedisDatabaseService::new().await.unwrap();
        let mut store = TokenStore {
            connection: db.connection,
        };

        let user_id = UserID::new();
        let token = store.create_token_for_user_id(&user_id).await;
        assert_eq!(user_id, store.get_user_id_of_token(&token).await.unwrap());
    }
}
