use crate::model::message::MessageToken;
use crate::model::user::{User, UserID};

use crate::service::token::random_alphanumeric_string;
use log::{error, info};
use redis::aio::MultiplexedConnection;
use redis::{aio, AsyncCommands};
use std::str::FromStr;
use tokio::sync::Mutex;

const TOKEN_LENGTH: u32 = 32;

pub struct TokenStore {
    pub connection: MultiplexedConnection,
}

impl TokenStore {
    pub async fn create_token_for_user_id(&mut self, user_id: &UserID) -> MessageToken {
        info!("create token for user_id {}", user_id);
        let token: String = random_alphanumeric_string(TOKEN_LENGTH);
        let key_user_id_to_token: String = format!("user_id_to_token:{user_id}");
        let _: u8 = self
            .connection
            .sadd(&key_user_id_to_token, &token)
            .await
            .map_err(|e| error!("{}", e))
            .unwrap();

        let key_token_to_user_id: String = format!("token_to_user_id:{token}");
        let _: u8 = self
            .connection
            .hset(key_token_to_user_id, "user_id", &user_id.to_string())
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
    use needs_env_var::needs_env_var;
    use crate::RedisDatabaseService;


    #[tokio::test]
    async fn test_token_store() {
        let db = RedisDatabaseService::new().await.unwrap();
        let mut store = TokenStore {
            connection: db.connection,
        };
        let user_id = UserID::new();
        store.create_token_for_user_id(&user_id);
        store.create_token_for_user_id(&user_id);
        let retrieved = store.get_token_of_user_id(&user_id);
        // assert_eq!(retrieved.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_user_id_of_token() {
        let db = RedisDatabaseService::new().await.unwrap();
        let mut store = TokenStore {
            connection: db.connection,
        };

        let user_id = UserID::new();
        store.create_token_for_user_id(&user_id);
        let token = store.create_token_for_user_id(&user_id);
        // assert_eq!(&user_id, store.get_user_id_of_token(&token).unwrap());
    }
}
