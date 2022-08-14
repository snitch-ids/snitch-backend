use crate::model::MessageBackend;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio;
use crate::persistance::Persist;

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

    async fn add_message(&mut self, message: MessageBackend) -> Result<()> {
        let _: () = self.connection.set(&message.hostname, &message).await?;
        Ok(())
    }

    async fn find_messages(&mut self, hostname: String) -> Result<Vec<MessageBackend>> {
        let response: String = self.connection.get(hostname).await?;
        let message: MessageBackend = serde_json::from_str(&response)?;
        Ok(vec![message])
    }
}

#[tokio::test]
async fn my_test() {
    use snitch::test_utils::get_test_message;

    let mut db_service = RedisDatabaseService::new("redis://127.0.0.1:6379")
        .await
        .expect("failed to connect to redis server");
    println!("START!");
    let message: MessageBackend = get_test_message().into();
    let hostname = message.hostname.clone();
    db_service.add_message(message).await.unwrap();
    let messages = db_service.find_messages(hostname).await.unwrap();
    println!("found messages: {:?}", messages);
}
