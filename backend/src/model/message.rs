use actix_web::cookie::time::macros::time;
use chatterbox::message::{Message as ChatterboxMessage, Notification};
use chrono::{DateTime, Utc};
use prost::{Enumeration, Message};
use rdkafka::message::ToBytes;
use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::Cursor;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::SystemTime;

pub type MessageToken = String;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/greeter.rs"));
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct MessageBackend {
    pub hostname: String,
    pub title: String,
    pub body: String,
    pub timestamp: Option<DateTime<Utc>>,
}

impl From<MessageBackend> for ProtoMessageBackend {
    fn from(value: MessageBackend) -> Self {
        let timestamp = value.timestamp.unwrap();
        let s: SystemTime = timestamp.into();
        let timestamp = prost_types::Timestamp::from(s);
        Self {
            hostname: value.hostname,
            title: value.title,
            body: value.body,
            timestamp: Some(timestamp),
        }
    }
}

impl From<ProtoMessageBackend> for MessageBackend {
    fn from(value: ProtoMessageBackend) -> Self {
        let timestamp = value.timestamp.unwrap();
        let s = chrono::DateTime::<Utc>::from_timestamp(timestamp.seconds, timestamp.nanos as u32);

        Self {
            hostname: value.hostname,
            title: value.title,
            body: value.body,
            timestamp: s,
        }
    }
}

pub type ProtoMessageBackend = proto::BackendMessage;

impl From<&String> for ProtoMessageBackend {
    fn from(s: &String) -> Self {
        ProtoMessageBackend::decode(&mut Cursor::new(s)).unwrap()
    }
}

impl Serialize for ProtoMessageBackend {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let as_string = serde_json::to_string(self).expect("failed parsing to string");
        serializer.serialize_str(&as_string)
    }
}

pub fn serialize_message(message: &ProtoMessageBackend) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(message.encoded_len());
    message.encode(&mut buf).unwrap();
    buf
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
    fn message(&self) -> ChatterboxMessage {
        let body = self.body.clone() + "\n\n" + &self.hostname;
        ChatterboxMessage {
            title: self.title.clone(),
            body,
        }
    }
}
