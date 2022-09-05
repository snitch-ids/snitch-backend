use crate::model::MessageBackend;
use crate::persistance::Persist;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use log::info;
use redis::aio;
use redis::AsyncCommands;

pub struct RedisDatabaseService {
    pub client: redis::Client,
    pub connection: aio::Connection,
}

impl RedisDatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let connection = client.get_async_connection().await?;
        Ok(RedisDatabaseService { client, connection })
    }
}

#[async_trait]
impl Persist for RedisDatabaseService {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()> {
        info!("storing in database: {:?}", message);
        let _: () = self
            .connection
            .rpush(&message.hostname, message)
            .await
            .unwrap();
        info!("storing in database: {:?}... finished", message);

        Ok(())
    }

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>> {
        let responses: Vec<String> = self.connection.lrange(hostname, 0, -2).await?;
        let messages = responses
            .iter()
            .map(|response| serde_json::from_str(response).unwrap())
            .collect();
        Ok(messages)
    }
}

#[tokio::test]
async fn redis_test() {
    use snitch::test_utils::get_test_message;

    let mut db_service = RedisDatabaseService::new("redis://127.0.0.1:6379")
        .await
        .expect("failed to connect to redis server");
    println!("START!");
    let message: MessageBackend = get_test_message().into();
    let hostname = message.hostname.as_ref();
    println!("test message {:?}", message);
    db_service.add_message(&message).await.unwrap();
    let messages = db_service.find_messages(hostname).await.unwrap();
    println!("found messages: {:?}", messages);
}
