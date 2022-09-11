use redis::{FromRedisValue, RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    token: Vec<String>,
}

impl ToRedisArgs for User {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let as_string = serde_json::to_string(self).expect("failed parsing to string");
        out.write_arg(as_string.as_bytes());
    }
}

impl FromRedisValue for User {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        println!("value...: {:?}", v);
        Ok(User {
            id: 123,
            email: "x@x.x".to_owned(),
            password: "xxx".to_owned(),
            token: vec!["xxx".to_owned()],
        })
    }
}
