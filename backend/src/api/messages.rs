use crate::api::AppState;
use crate::model::message::{MessageBackend, MessageToken};
use crate::persistence::{MessageKey, PersistMessage};
use actix::Addr;
use actix_identity::Identity;
use actix_web::{get, post, web, Responder};

use crate::errors::APIError;

use crate::model::user::UserID;
use crate::TokenState;
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::service::kafka::{KafkaActor, TryNotify as KafkaNotify};
use crate::service::notification_dispatcher::{NotificationActor, TryNotify};
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
    kafka_add: web::Data<Addr<KafkaActor>>,
) -> Result<impl Responder, APIError> {
    let mut token_store = token_state.token.lock().await;
    let token: MessageToken = auth.token().trim().to_string();
    match token_store.get_user_id_of_token(&token).await {
        None => {
            info!("no user id of token {token}");
            return Err(APIError::Unauthorized);
        }
        Some(user_id) => {
            kafka_add.do_send(KafkaNotify(user_id, message.0));
        }
    }

    Ok("success".to_string())
}

#[get("/hostnames")]
pub(crate) async fn get_message_hostnames(
    identity: Identity,
    state: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    let mut messages_state = state.persist.lock().await;
    let user_id: UserID = identity.id().unwrap().into();

    let hostnames: Vec<String> = messages_state
        .get_hostnames_of_user(&user_id)
        .await
        .map_err(|_| APIError::InternalServerError)?;
    info!("returning {} objects ", hostnames.len());
    Ok(web::Json(hostnames))
}

#[get("/messages/{hostname}")]
pub(crate) async fn get_messages_by_hostname(
    path: web::Path<String>,
    identity: Identity,
    state: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    let hostname = path.into_inner();
    info!("received request for {}", &hostname);
    let user_id: UserID = identity.id().unwrap().into();
    let key = MessageKey { user_id, hostname };
    let mut messages_state = state.persist.lock().await;
    let messages: Vec<MessageBackend> = messages_state
        .find_messages(&key)
        .await
        .map_err(|_| APIError::InternalServerError)?;
    info!("returning {} objects ", messages.len());
    Ok(web::Json(messages))
}
