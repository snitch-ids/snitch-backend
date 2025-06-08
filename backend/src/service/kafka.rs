use crate::persistence::redis::NotificationSettings;
use actix::ActorFutureExt;
use actix::{Actor, Context, Handler, Message as ActixMessage, ResponseActFuture, WrapFuture};
use chatterbox::message::Dispatcher;
use rdkafka::Message;
use std::io::Cursor;

use crate::api::AppState;
use crate::model::message::{serialize_message, MessageBackend, MessageToken, ProtoMessageBackend};
use crate::model::user::UserID;
use crate::persistence::{MessageKey, PersistMessage};
use actix_web::web::Data;
use log::{info, warn};
use prost::Message as _;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer,
};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Header, Headers, OwnedHeaders, ToBytes};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;
use rdkafka::{ClientContext, TopicPartitionList};
use serde_json::ser::State;
use std::time::Duration;
use tokio::task::JoinHandle;

async fn produce(brokers: &str, topic_name: &str) {
    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        info!("Future completed. Result: {:?}", future.await);
    }
}

#[derive(Clone)]
pub(crate) struct KafkaManager {
    producer: FutureProducer,
}

impl KafkaManager {
    pub(crate) fn new() -> KafkaManager {
        let producer: FutureProducer = ClientConfig::new()
            // .set("sasl.mechanism", "SCRAM-SHA-256")
            // .set("sasl.username", "tool")
            // .set("sasl.password", "token")
            .set("group.id", "snitch-backend")
            .set("bootstrap.servers", "localhost:9092")
            .set("queue.buffering.max.ms", "1000")
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Self { producer }
    }
}

impl KafkaManager {
    pub(crate) async fn try_notify(&self, message: TryNotify) -> bool {
        let payload = serialize_message(&message.1);
        let delivery_status = self
            .producer
            .send(
                FutureRecord::to("messages-backend")
                    .payload(&payload)
                    .key(&message.0)
                    .headers(OwnedHeaders::new().insert(Header {
                        key: "header_key",
                        value: Some("header_value"),
                    })),
                Duration::from_secs(0),
            )
            .await;
        delivery_status.is_ok()
    }
}

#[derive(ActixMessage, Clone)]
#[rtype(result = "Result<bool, ()>")]
pub(crate) struct TryNotify(pub UserID, pub ProtoMessageBackend);

pub(crate) struct KafkaActor {
    pub(crate) producer: KafkaManager,
}

impl KafkaActor {
    pub(crate) fn new(producer: KafkaManager) -> Self {
        Self { producer }
    }
}

impl Actor for KafkaActor {
    type Context = Context<Self>;
}

impl Handler<TryNotify> for KafkaActor {
    type Result = ResponseActFuture<Self, Result<bool, ()>>;

    fn handle(&mut self, msg: TryNotify, _: &mut Context<Self>) -> Self::Result {
        println!("trying to notify: {:?}", msg.0);
        let p = self.producer.clone();
        Box::pin(
            async move {
                p.try_notify(msg).await;
                Ok(true)
            }
            .into_actor(self),
        )
    }
}

pub(crate) struct KafkaPersistClient {
    handle: JoinHandle<()>,
}

impl KafkaPersistClient {
    pub(crate) fn new(state: Data<AppState>) -> Self {
        let handle = tokio::task::spawn(async move {
            consume_and_store(
                "localhost:9092",
                "snitch-backend",
                &["messages-backend"],
                None,
                state,
            )
            .await;
        });
        KafkaPersistClient { handle }
    }
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

async fn consume_and_store(
    brokers: &str,
    group_id: &str,
    topics: &[&str],
    assignor: Option<&String>,
    state: Data<AppState>,
) {
    let context = CustomContext;

    let mut config = ClientConfig::new();

    config
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug);

    if let Some(assignor) = assignor {
        config
            .set("group.remote.assignor", assignor)
            .set("group.protocol", "consumer")
            .remove("session.timeout.ms");
    }

    let consumer: LoggingConsumer = config
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(topics)
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<[u8]>() {
                    None => None,
                    Some(Ok(s)) => {
                        let message = ProtoMessageBackend::decode(&mut Cursor::new(s)).unwrap();
                        Some(message)
                    }
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        None
                    }
                };
                let message = payload.unwrap();

                let user_id = m.key().unwrap();
                let user_id = UserID(String::from_utf8(user_id.to_vec()).unwrap());
                info!(
                    "key: '{:?}', message: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    user_id,
                    message,
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
                let message_key = MessageKey {
                    user_id,
                    hostname: message.hostname.clone(),
                };
                let message = MessageBackend::from(message);
                state
                    .persist
                    .lock()
                    .await
                    .add_message(&message_key, &message)
                    .await
                    .unwrap();
            }
        };
    }
}
