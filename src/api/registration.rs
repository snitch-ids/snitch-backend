use crate::api::AppStateWithCounter;

use actix_web::post;
use actix_web::web::Data;
use actix_web::{web, Responder};

use log::info;
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
use reqwest::Url;

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

    let activation_link = Url::parse(&format!("{REPLY_URL}/{nonce}")).unwrap();
    let mail = generate_registration_mail("", &activation_link);
    let receiver = user.username.parse().unwrap();
    send_registration_mail(mail, receiver).await;
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
    "registered".to_string()
}
