use crate::api::AppStateWithCounter;
use std::env;
use std::fmt::format;

use actix_web::web::Data;
use actix_web::{post, HttpResponse};
use actix_web::{web, Responder};

use log::{error, info};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegistrationRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

use crate::model::user::{Nonce, User};
use crate::service::email::{generate_registration_mail, send_registration_mail};
use crate::service::token::random_alphanumeric_string;
use actix_web::get;
use lettre::message::Mailbox;
use reqwest::Url;

#[post("/register")]
pub async fn register(
    register_request: web::Json<RegistrationRequest>,
    state: Data<AppStateWithCounter>,
) -> String {
    info!("register");
    let nonce = random_alphanumeric_string(40);
    let backend_url = env::var("SNITCH_BACKEND_URL").expect("SNITCH_BACKEND_URL not defined");
    let activation_link = Url::parse(&format!("{backend_url}/register/{nonce}")).unwrap();
    let mail = generate_registration_mail("", &activation_link);

    let mut users = state.messages.lock().await;
    let user_request = register_request.into_inner();
    let user = User::from(user_request);
    let receiver: Mailbox = user.username.parse().unwrap();

    match users.add_user_pending(&user, &nonce).await {
        Ok(_) => {
            send_registration_mail(mail, receiver).await;
        }
        Err(e) => {
            error!("failed adding pending user {}", e);
        }
    };

    "Sent mail".to_string()
}

#[get("/register/{nonce}")]
pub async fn register_reply(
    nonce: web::Path<Nonce>,
    state: Data<AppStateWithCounter>,
) -> impl Responder {
    let nonce = nonce.into_inner();
    let mut users = state.messages.lock().await;
    users.confirm_user_pending(&nonce).await.unwrap();
    let reply_url = env::var("SNITCH_FRONTEND_URL").expect("SNITCH_FRONTEND_URL undefined");

    HttpResponse::Found()
        .append_header(("Location", reply_url))
        .finish()
}
