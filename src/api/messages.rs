use crate::api::AppStateWithCounter;
use crate::model::message::{MessageBackend, MessageToken};
use crate::persistance::{MessageKey, PersistMessage};
use actix_identity::Identity;
use actix_web::{get, post, web, Responder};

use crate::errors::APIError;

use crate::model::user::UserID;
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

#[post("/messages")]
pub(crate) async fn add_message(
    auth: BearerAuth,
    message: web::Json<MessageBackend>,
    token_state: web::Data<TokenState>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, APIError> {
    let mut token_store = token_state.token.lock().await;
    let token: MessageToken = auth.token().trim().to_string();

    let obj = message.into_inner();
    let mut message_db = state.messages.lock().await;
    match token_store.get_user_id_of_token(&token).await {
        None => {
            info!("no user id of token {token}")
        }
        Some(user_id) => {
            let key = MessageKey {
                user_id: user_id.clone(),
                hostname: obj.hostname.clone(),
            };
            message_db
                .add_message(&key, &obj)
                .await
                .expect("failed adding message");
        }
    }

    Ok("success".to_string())
}

#[get("/messages/hostnames")]
pub(crate) async fn get_message_hostnames(
    identity: Identity,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, APIError> {
    let mut messages_state = state.messages.lock().await;
    let user_id: UserID = identity.id().unwrap().into();

    let hostnames: Vec<String> = messages_state
        .get_hostnames_of_user(&user_id)
        .await
        .map_err(|_| APIError::InternalServerError)?;
    info!("returning {} objects ", hostnames.len());
    Ok(web::Json(hostnames))
}

#[post("/messages/all")]
pub(crate) async fn get_messages_by_hostname(
    identity: Identity,
    info: web::Json<MessageRequest>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, APIError> {
    info!("received request for {}", &info.hostname);
    let mut messages_state = state.messages.lock().await;
    let user_id: UserID = identity.id().unwrap().into();
    let key = MessageKey {
        user_id,
        hostname: info.hostname.clone(),
    };
    let messages: Vec<MessageBackend> = messages_state
        .find_messages(&key)
        .await
        .map_err(|_| APIError::InternalServerError)?;
    info!("returning {} objects ", messages.len());
    Ok(web::Json(messages))
}
