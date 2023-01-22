use crate::api::AppStateWithCounter;
use crate::persistance::token::TokenState;
use actix_identity::Identity;
use actix_web::http::StatusCode;
use actix_web::post;
use actix_web::web::{Data, Json};
use actix_web::{web, HttpMessage, Responder, Scope};
use actix_web_lab::web::Redirect;
use log::{debug, info};
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::string::ToString;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
pub struct RegistrationRequest {
    username: String,
    password: String,
}

use crate::service::token::random_alphanumeric_string;
use actix_web::{get, App, HttpServer};

type Nonce = String;

pub struct PendingUsersState {
    pub(crate) users: Mutex<HashMap<Nonce, RegistrationRequest>>,
}

impl PendingUsersState {
    pub fn new() -> Self {
        let empty_pending_state: HashMap<Nonce, RegistrationRequest> = HashMap::new();
        PendingUsersState {
            users: Mutex::new(empty_pending_state),
        }
    }
}

const REPLY_URL: &str = "http://localhost:8081/register";

#[post("/register")]
pub async fn register(
    register_request: web::Json<RegistrationRequest>,
    pendin_users_state: web::Data<PendingUsersState>,
) -> String {
    let mut users = pendin_users_state.users.lock().unwrap();
    let nonce = random_alphanumeric_string(40);
    users.insert(nonce.clone(), register_request.into_inner());
    format!("{}/{}", REPLY_URL, nonce)
}

#[get("/register/{nonce}")]
pub async fn register_reply(
    nonce: web::Path<Nonce>,
    state: Data<AppStateWithCounter>,
    pendin_users_state: Data<PendingUsersState>,
) -> impl Responder {
    let nonce = nonce.into_inner();
    let mut users = pendin_users_state.users.lock().unwrap();
    let confirmed = users.get(&nonce).unwrap();
    // let users_pending = pending_users.user_store.lock().await;
    info!("register {:?}", confirmed);
    format!("done {}", nonce)
}
