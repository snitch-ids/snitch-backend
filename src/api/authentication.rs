
use crate::AppStateWithCounter;
use actix_identity::Identity;



use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{error, get, post, web, HttpMessage, Responder};
use actix_web_lab::web::Redirect;
use log::{debug};
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
) -> impl Responder {
    let users = state.users.lock().await;
    match users.valid_password(&login_request.username, &login_request.password) {
        true => {
            let user = users
                .get_user_by_name(&login_request.username)
                .expect("failed getting user");
            Identity::login(&req.extensions(), user.user_id.to_string()).unwrap();
            Redirect::to("/").using_status_code(StatusCode::FOUND)
        }
        false => {
            debug!(
                "invalid username {} {}",
                login_request.username, login_request.password
            );
            Redirect::to("/login").using_status_code(StatusCode::NOT_FOUND)
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
