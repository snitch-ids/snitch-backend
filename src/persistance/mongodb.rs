use anyhow::{Ok, Result};
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};
use mongodb::{options::ClientOptions, Client};
use snitch::notifiers::Message;

pub struct DatabaseService {
    pub client: Client,
}

impl DatabaseService {
    pub async fn new(url: &str) -> Result<Self> {
        // Parse a connection string into an options struct.
        let mut client_options = ClientOptions::parse(url).await?;

        client_options.app_name = Some("Snitch".to_string());
        let client = Client::with_options(client_options)?;

        Ok(DatabaseService { client })
    }

    pub async fn add_message(&self, message: Message) -> Result<()> {
        let db = self.client.database("snitch");
        let typed_collection = db.collection::<Message>("messages");
        typed_collection.insert_one(message, None).await?;
        Ok(())
    }

    pub async fn find_messages(&self, hostname: String) -> Result<Vec<Message>> {
        let db = self.client.database("snitch");
        let typed_collection = db.collection::<Message>("messages");
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

    let db_service = DatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
        .await
        .unwrap();

    let message = get_test_message();
    let hostname = "Mariuss-MacBook-Air.local".to_owned();
    db_service.add_message(message).await;
    let messages = db_service.find_messages(hostname).await.unwrap();
    println!("found messages: {:?}", messages);
}
