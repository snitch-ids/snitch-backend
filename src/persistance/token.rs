use crate::model::message::MessageToken;
use crate::model::user::UserID;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

const TOKEN_LENGTH: i32 = 32;

#[derive(Default)]
pub struct TokenStore {
    pub tokens: HashMap<UserID, HashSet<MessageToken>>,
}

impl TokenStore {
    pub fn create_token_for_user_id(&mut self, user_id: &UserID) -> MessageToken {
        let mut rng = rand::thread_rng();
        let user_token = self.tokens.entry(*user_id).or_default();
        let token: String = (0..TOKEN_LENGTH)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        user_token.insert(token.clone());
        token
    }

    pub fn get_token_of_user_id(&self, user_id: &UserID) -> Option<&HashSet<MessageToken>> {
        self.tokens.get(user_id)
    }
}

#[derive(Default)]
pub struct TokenState {
    pub token: Mutex<TokenStore>,
}

#[test]
fn test_token_store() {
    let mut store = TokenStore::default();
    let user_id = 0;
    store.create_token_for_user_id(&user_id);
    store.create_token_for_user_id(&user_id);
    let retrieved = store.get_token_of_user_id(&user_id);
    assert_eq!(retrieved.unwrap().len(), 2);
}
