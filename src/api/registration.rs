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

#[derive(Deserialize, Debug)]
pub struct RegistrationRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

use crate::api::users::AddUserRequest;
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
    let user = User::from(user_request);
    users.add_user_pending(&user, &nonce).await;
    format!("{}/{}", REPLY_URL, nonce)
}

#[get("/register/{nonce}")]
pub async fn register_reply(
    nonce: web::Path<Nonce>,
    state: Data<AppStateWithCounter>,
) -> impl Responder {
    let nonce = nonce.into_inner();
    let mut users = state.messages.lock().await;
    users.confirm_user_pending(&nonce).await.unwrap();
    format!("registered")
}
