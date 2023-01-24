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

use crate::model::user::{Nonce, User};
use crate::service::token::random_alphanumeric_string;
use actix_web::{get, App, HttpServer};

const REPLY_URL: &str = "http://localhost:8081/register";

#[post("/register")]
pub async fn register(
    register_request: web::Json<RegistrationRequest>,
    state: web::Data<AppStateWithCounter>,
) -> String {
    info!("register");
    let nonce = random_alphanumeric_string(40);
    let mut users = state.messages.lock().await;
    let user_request = register_request.into_inner();
    // users.insert(nonce.clone(),);
    let user = User::new(user_request.username, user_request.password);
    users.add_user(&user);
    format!("{}/{}", REPLY_URL, nonce)
}

#[get("/register/{nonce}")]
pub async fn register_reply(
    nonce: web::Path<Nonce>,
    state: Data<AppStateWithCounter>,
) -> impl Responder {
    let nonce = nonce.into_inner();
    let mut users = state.users.lock().await;
    // let confirmed = users.get_users().unwrap();
    // let users_pending = pending_users.user_store.lock().await;
    // info!("register {:?}", confirmed);
    format!("done {}", nonce)
}
