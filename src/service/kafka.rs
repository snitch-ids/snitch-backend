use crate::persistence::redis::NotificationSettings;
use actix::ActorFutureExt;
use actix::{Actor, Context, Handler, Message, ResponseActFuture, WrapFuture};
use chatterbox::message::Dispatcher;

use std::time::Duration;

use log::info;

use crate::model::message::{MessageBackend, MessageToken};
use crate::model::user::UserID;
use crate::persistence::MessageKey;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders, ToBytes};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;
use serde_json::ser::State;

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
        let delivery_status = self
            .producer
            .send(
                FutureRecord::to("messages-backend")
                    .payload(&message.1)
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

#[derive(Message, Clone)]
#[rtype(result = "Result<bool, ()>")]
pub(crate) struct TryNotify(pub UserID, pub MessageBackend);

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

struct KafkaDispatcher {
    state: State,
}

impl KafkaDispatcher {
    // async fn handle_message(&self, message: String) {
    //
    //     if self.state
    //         .notification_filter
    //         .lock()
    //         .await
    //         .notify_user(&user_id)
    //         .await
    //     {
    //         let notification_settings = self.state
    //             .persist
    //             .lock()
    //             .await
    //             .get_notification_settings(&user_id)
    //             .await;
    //     }
    //     let key = MessageKey {
    //         user_id,
    //         hostname: message.hostname.clone(),
    //     };
    //     self.state
    //         .persist
    //         .lock()
    //         .await
    //         .add_message(&key, &message)
    //         .await
    //         .expect("failed adding message");
    //
    // Ok("success".to_string())
    // }
}
