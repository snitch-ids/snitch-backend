use crate::api::AppStateWithCounter;
use crate::model::MessageBackend;
use crate::persistance::Persist;
use actix_web::{get, post, web, Responder};
use log::info;
use serde::Deserialize;
use serde::Serialize;

#[post("/messages/")]
async fn add_message(
    message: web::Json<MessageBackend>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("adding message");
    let obj = message.into_inner();
    let mut messages = state.messages.lock().await;
    messages
        .add_message(obj)
        .await
        .expect("failed adding message");
    format!("added message")
}

#[derive(Debug, Deserialize)]
struct MessageRequest {
    hostname: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    messages: Vec<MessageBackend>,
}

#[get("/messages/")]
async fn get_messages_by_hostname (
    info: web::Query<MessageRequest>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("get message by hostname");
    let mut messages_state = state.messages.lock().await;
    let messages: Vec<MessageBackend> = messages_state
        .find_messages(info.hostname.to_owned())
        .await
        .expect("failed retrieving message");

    web::Json(messages)
}
