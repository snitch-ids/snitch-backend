use crate::api::AppStateWithCounter;
use crate::model::MessageBackend;
use crate::persistance::Persist;
use actix_web::{get, post, web, Responder};
use log::info;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
struct MessageRequest {
    hostname: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    messages: Vec<MessageBackend>,
}

#[post("/messages/")]
async fn add_message(
    message: web::Json<MessageBackend>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("adding message");
    let obj = message.into_inner();
    let mut message_db = state.messages.lock().await;
    message_db
        .add_message(&obj)
        .await
        .expect("failed adding message");
    "added message".to_string()
}

#[get("/messages/")]
async fn get_messages_by_hostname(
    info: web::Query<MessageRequest>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("received request for {}", &info.hostname);
    let mut messages_state = state.messages.lock().await;
    let messages: Vec<MessageBackend> = messages_state
        .find_messages(&info.hostname)
        .await
        .expect("failed retrieving message");
    info!("found {} entires", messages.len());
    web::Json(messages)
}
