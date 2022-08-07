use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub hostname: String,
    pub timestamp: String,
    pub title: String,
    pub content: String,
}
