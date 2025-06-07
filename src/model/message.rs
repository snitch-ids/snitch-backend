use chatterbox::message::{Message, Notification};
use chrono::{DateTime, Utc};
use rdkafka::message::ToBytes;
use redis::{RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub type MessageToken = String;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MessageBackend {
    pub hostname: String,
    pub title: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip)]
    cached_bytes: OnceLock<Vec<u8>>,
}

impl MessageBackend {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).expect("failed parsing from bytes")
    }
}

impl ToBytes for MessageBackend {
    fn to_bytes(&self) -> &[u8] {
        let as_bytes = self.cached_bytes.get_or_init(|| {
            let as_string = serde_json::to_string(self).expect("failed parsing to string");
            as_string.as_bytes().to_vec()
        });
        as_bytes
    }
}

impl ToRedisArgs for MessageBackend {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let as_string = serde_json::to_string(self).expect("failed parsing to string");
        out.write_arg(as_string.as_bytes());
    }
}

impl Notification for MessageBackend {
    fn message(&self) -> Message {
        let body = self.body.clone() + "\n\n" + &self.hostname;
        Message {
            title: self.title.clone(),
            body,
        }
    }
}
