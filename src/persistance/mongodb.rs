use crate::model::MessageBackend;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};
use mongodb::{options::ClientOptions, Client};

use crate::persistance::Persist;

pub struct MongoDatabaseService {
    pub client: Client,
}

impl MongoDatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;

        client_options.app_name = Some("Snitch".to_string());
        let client = Client::with_options(client_options)?;

        Ok(MongoDatabaseService { client })
    }
}

#[async_trait]
impl Persist for MongoDatabaseService {
    async fn add_message(&mut self, message: &MessageBackend) -> Result<()> {
        let db = self.client.database("snitch");
        let typed_collection = db.collection::<MessageBackend>("messages");
        typed_collection.insert_one(message, None).await?;
        Ok(())
    }

    async fn find_messages(&mut self, hostname: &str) -> Result<Vec<MessageBackend>> {
        let db = self.client.database("snitch");
        let typed_collection = db.collection::<MessageBackend>("messages");
        let filter = doc! { "hostname": hostname };
        let find_options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();
        let mut cursor = typed_collection.find(filter, find_options).await?;

        let mut results = vec![];
        // Iterate over the results of the cursor.
        while let Some(message) = cursor.try_next().await? {
            results.push(message);
        }
        Ok(results)
    }
}

#[tokio::test]
async fn my_test() {
    use snitch::test_utils::get_test_message;

    let mut db_service = MongoDatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
        .await
        .unwrap();

    let message: MessageBackend = get_test_message().into();
    let hostname = "Mariuss-MacBook-Air.local";
    db_service.add_message(&message).await;
    let messages = db_service.find_messages(hostname).await.unwrap();
    println!("found messages: {messages:?}");
}
