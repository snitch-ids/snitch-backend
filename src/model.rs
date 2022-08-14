use std::ops::{Deref, DerefMut};

use redis::{from_redis_value, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};
use snitch::notifiers::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageBackend(Message);

impl ToRedisArgs for MessageBackend {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let as_string = serde_json::to_string(self).expect("failed parsing to string");
        out.write_arg(as_string.as_bytes());
    }
}

impl Deref for MessageBackend {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageBackend {
    fn deref_mut(&mut self) -> &mut Message {
        &mut self.0
    }
}

impl From<Message> for MessageBackend {
    fn from(msg: Message) -> MessageBackend {
        MessageBackend(msg)
    }
}
