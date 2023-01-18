use crate::model::user::User;
use crate::AppStateWithCounter;
use actix_jwt_auth_middleware::{AuthError, AuthResult, CookieSigner};
use actix_session::Session;
use actix_web::cookie::Cookie;
use actix_web::http::header::LOCATION;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse};
use jwt_compact::alg::Ed25519;
use log::{debug, info};
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
    login_request: web::Json<LoginRequest>,
    state: Data<AppStateWithCounter>,
    cookie_signer: Data<CookieSigner<User, Ed25519>>,
    session: Session,
) -> AuthResult<HttpResponse> {
    let users = state.users.lock().await;
    match users.valid_password(&login_request.username, &login_request.password) {
        true => {
            let user = users
                .get_user_by_name(&login_request.username)
                .expect("failed getting user");

            Ok(
                HttpResponse::Ok()
                    // .insert_header((LOCATION, "http://127.0.0.1:8080/messages/"))
                    .finish(), // .cookie(access_token)
                               // .cookie(refresh_token)
                               // .json(cookies)
            )
        }
        false => {
            debug!(
                "invalid username {} {}",
                login_request.username, login_request.password
            );
            Err(AuthError::NoCookie)
        }
    }
}

#[get("/hello")]
pub async fn hello(user: User) -> impl actix_web::Responder {
    info!("hi");
    format!("Hello there, i see your user id is {}.", user.username)
}
