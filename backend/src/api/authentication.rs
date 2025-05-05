use crate::AppState;
use actix_identity::Identity;

use crate::service::authentication::valid_hash;
use validator::Validate;

use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{error, get, post, web, HttpMessage, Responder};

use actix_web::web::Redirect;

use crate::errors::APIError;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8, max = 64))]
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

#[post("/login")]
pub async fn login(
    req: actix_web::HttpRequest,
    login_request: web::Json<LoginRequest>,
    state: Data<AppState>,
) -> Result<impl Responder, APIError> {
    let login_request = login_request.into_inner();
    if let Err(e) = login_request.validate() {
        return Err(APIError::BadRequest(format!("{e}")));
    }

    let mut users = state.persist.lock().await;
    let email = &login_request.email;
    debug!("login request for {}", email);
    if let Some(user) = users.get_user_by_email(email).await {
        if valid_hash(&user.password_hash, &login_request.password) {
            Identity::login(&req.extensions(), user.user_id.to_string()).unwrap();
            return Ok(user.email);
        }
    }

    Err(APIError::Unauthorized)
}

#[get("/")]
pub async fn index(identity: Option<Identity>) -> actix_web::Result<impl Responder> {
    let id = match identity.map(|id| id.id()) {
        None => "anonymous".to_owned(),
        Some(Ok(id)) => id,
        Some(Err(err)) => return Err(error::ErrorInternalServerError(err)),
    };

    Ok(format!("Hello {id}"))
}

#[post("/logout")]
pub async fn logout(id: Identity) -> impl Responder {
    info!("logging out {:?}", id.id());
    id.logout();
    Redirect::to("/").using_status_code(StatusCode::FOUND)
}
