use crate::api::AppStateWithCounter;
use crate::model::message::{MessageBackend, MessageToken};
use crate::persistance::Persist;
use actix_identity::Identity;
use actix_web::{post, web, Responder};

use crate::errors::ServiceError;

use crate::TokenState;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use log::info;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    hostname: String,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    messages: Vec<MessageBackend>,
}

#[post("/messages/")]
pub(crate) async fn add_message(
    auth: BearerAuth,
    message: web::Json<MessageBackend>,
    token_state: web::Data<TokenState>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let token_store = token_state.token.lock().await;
    let token: MessageToken = auth.token().trim().to_string();

    if !token_store.has_token(token) {
        return Err(ServiceError::BadRequest("invalid token".to_string()));
    }
    let obj = message.into_inner();
    let mut message_db = state.messages.lock().await;
    message_db
        .add_message(&obj)
        .await
        .expect("failed adding message");
    Ok("success".to_string())
}

#[post("/messages/all")]
pub(crate) async fn get_messages_by_hostname(
    _identity: Identity,
    info: web::Json<MessageRequest>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let _messages: Vec<MessageBackend> = vec![];
    info!("received request for {}", &info.hostname);
    let mut messages_state = state.messages.lock().await;
    let messages: Vec<MessageBackend> = messages_state
        .find_messages(&info.hostname)
        .await
        .map_err(|_| ServiceError::InternalServerError)?;
    info!("returning {} objects ", messages.len());
    Ok(web::Json(messages))
}
