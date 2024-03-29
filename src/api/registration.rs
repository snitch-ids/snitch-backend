use crate::api::AppStateWithCounter;
use std::env;
use std::fmt::format;
use validator::{Validate, ValidationError};

use actix_web::web::Data;
use actix_web::{post, HttpResponse};
use actix_web::{web, Responder};

use log::{error, info};
use serde::Deserialize;

use crate::errors::APIError::{BadRequest, InternalServerError};
use crate::model::user::{Nonce, User};
use crate::service::email::{generate_registration_mail, send_registration_mail};
use crate::service::token::random_alphanumeric_string;
use actix_web::get;
use lettre::message::Mailbox;
use reqwest::Url;

#[derive(Deserialize, Debug, Validate)]
pub struct RegistrationRequest {
    #[validate(email)]
    pub(crate) email: String,

    #[validate(length(min = 8, max = 64))]
    pub(crate) password: String,
}

#[post("/register")]
pub async fn register(
    register_request: web::Json<RegistrationRequest>,
    state: Data<AppStateWithCounter>,
) -> impl Responder {
    info!("register");

    let user_request = register_request.into_inner();
    if let Err(e) = user_request.validate() {
        return Err(BadRequest(format!("{e}")));
    }

    let nonce = random_alphanumeric_string(40);
    let backend_url = env::var("SNITCH_BACKEND_URL").expect("SNITCH_BACKEND_URL not defined");
    let activation_link = Url::parse(&format!("{backend_url}/register/{nonce}")).unwrap();
    let mail = generate_registration_mail("", &activation_link);

    let mut users = state.messages.lock().await;

    let user = User::from(user_request);
    let receiver: Mailbox = user.email.parse().unwrap();

    if let Err(e) = users.add_user_pending(&user, &nonce).await {
        error!("failed adding pending user {}", e);
    }

    if let Err(e) = send_registration_mail(mail, receiver).await {
        error!("{e}");
        return Err(InternalServerError);
    }
    Ok("ok")
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

#[cfg(test)]
mod test {
    use super::RegistrationRequest;
    use validator::Validate;

    #[test]
    fn test_validation() {
        let invalid_registrations = vec![
            RegistrationRequest {
                email: "".to_string(),
                password: "".to_string(),
            },
            RegistrationRequest {
                email: "md".to_string(),
                password: "".to_string(),
            },
            RegistrationRequest {
                email: "m.x@d.d".to_string(),
                password: "".to_string(),
            },
        ];

        for reg in invalid_registrations.iter() {
            assert!(reg.validate().is_err());
        }

        let valid = RegistrationRequest {
            email: "m.x@d.d".to_string(),
            password: "kdifjwelijsdf".to_string(),
        };
        assert!(valid.validate().is_ok());
    }
}
