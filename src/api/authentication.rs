use crate::AppStateWithCounter;
use actix_identity::Identity;

use crate::service::authentication::valid_hash;
use actix_web::error::ErrorUnauthorized;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{error, get, post, web, HttpMessage, Responder};
use actix_web_lab::__reexports::tracing::field::debug;
use actix_web_lab::web::Redirect;
use argonautica::Error;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
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
    state: Data<AppStateWithCounter>,
) -> actix_web::Result<impl Responder> {
    let mut users = state.messages.lock().await;
    let username = &login_request.username;
    let user = users
        .get_user_by_name(username)
        .await
        .map_err(|e| ErrorUnauthorized(e))?;
    match valid_hash(&user.password_hash, &login_request.password) {
        Ok(b) => {
            Identity::login(&req.extensions(), user.user_id.to_string()).unwrap();
            Ok(format!("username"))
        }
        Err(e) => {
            debug!("invalid username {username}",);
            Err(error::ErrorUnauthorized(e))
        }
    }
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
    id.logout();
    Redirect::to("/").using_status_code(StatusCode::FOUND)
}
