use crate::model::message::MessageToken;
use crate::model::user::UserID;

use std::collections::{HashMap, HashSet};

use crate::service::token::random_alphanumeric_string;
use tokio::sync::Mutex;

const TOKEN_LENGTH: u32 = 32;

#[derive(Default)]
pub struct TokenStore {
    pub tokens: HashMap<UserID, HashSet<MessageToken>>,
}

impl TokenStore {
    fn new() -> Self {
        let mut token_store = Self::default();

        #[cfg(debug_assertions)] // insert debug token for development
        {
            let id = UserID::new();
            token_store
                .tokens
                .entry(id)
                .or_default()
                .insert("!!!INSECUREADMINTOKEN!!!".to_string());
        }

        token_store
    }

    pub fn create_token_for_user_id(&mut self, user_id: &UserID) -> MessageToken {
        let token = random_alphanumeric_string(TOKEN_LENGTH);
        let user_token = self.tokens.entry(user_id.clone()).or_default();
        user_token.insert(token.clone());

        token
    }

    pub fn get_token_of_user_id(&self, user_id: &UserID) -> Option<&HashSet<MessageToken>> {
        self.tokens.get(user_id)
    }

    pub fn get_user_id_of_token(&self, token: &MessageToken) -> Option<&UserID> {
        for (user_id, tokens) in self.tokens.iter() {
            if tokens.contains(token) {
                return Some(user_id);
            }
        }

        None
    }

    pub fn has_token(&self, token: &MessageToken) -> bool {
        for x in self.tokens.values() {
            let value = x.contains(token);
            if value {
                return true;
            }
        }
        false
    }
}

#[derive(Default)]
pub struct TokenState {
    pub token: Mutex<TokenStore>,
}

impl TokenState {
    pub fn new() -> Self {
        Self {
            token: Mutex::new(TokenStore::new()),
        }
    }
}

#[test]
fn test_token_store() {
    let mut store = TokenStore::new();
    let user_id = UserID::new();
    store.create_token_for_user_id(&user_id);
    store.create_token_for_user_id(&user_id);
    let retrieved = store.get_token_of_user_id(&user_id);
    assert_eq!(retrieved.unwrap().len(), 2);
}

#[test]
fn test_store_has_token() {
    let mut store = TokenStore::default();
    let user_id = UserID::new();
    let token = store.create_token_for_user_id(&user_id);
    assert!(store.has_token(&token));
    assert!(!store.has_token(&"NONEXISTENDTOKEN".to_string()));
}

#[test]
fn test_user_id_of_token() {
    let mut store = TokenStore::default();
    let user_id = UserID::new();
    store.create_token_for_user_id(&user_id);
    let token = store.create_token_for_user_id(&user_id);
    assert_eq!(&user_id, store.get_user_id_of_token(&token).unwrap());
}
