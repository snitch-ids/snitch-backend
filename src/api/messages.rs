use crate::api::AppStateWithCounter;
use actix_web::{get, post, web, Responder};
use log::info;
use serde::Deserialize;
use serde::Serialize;
use snitch::notifiers::Message;

#[post("/messages/")]
async fn add_message(
    message: web::Json<Message>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("adding message");
    let obj = message.into_inner();
    let messages = state.messages.lock().await;
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
    messages: Vec<Message>,
}

#[get("/messages/")]
async fn get_messages_by_hostname(
    info: web::Query<MessageRequest>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("get message by hostname");
    let messages_state = state.messages.lock().await;
    let messages: Vec<Message> = messages_state
        .find_messages(info.hostname.to_owned())
        .await
        .expect("failed retrieving message");

    web::Json(messages)
}
