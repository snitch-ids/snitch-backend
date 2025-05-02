use std::ffi::OsString;
use chrono::{DateTime, Utc};

use redis::{RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};

pub type MessageToken = String;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MessageBackend {
    pub hostname: OsString,
    pub title: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
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
