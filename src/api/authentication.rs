use crate::model::user::User;
use crate::AppStateWithCounter;
use actix_jwt_auth_middleware::{AuthError, AuthResult, CookieSigner};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse};
use jwt_compact::alg::Ed25519;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn login(
    login_request: web::Json<LoginRequest>,
    state: Data<AppStateWithCounter>,
    cookie_signer: Data<CookieSigner<User, Ed25519>>,
) -> AuthResult<HttpResponse> {
    info!("login request");
    let users = state.users.lock().await;
    match users.valid_password(&login_request.username, &login_request.password) {
        true => {
            let user = users
                .get_user_by_name(&login_request.username)
                .expect("failed getting user");
            Ok(HttpResponse::Ok()
                .cookie(cookie_signer.create_access_token_cookie(user)?)
                .cookie(cookie_signer.create_refresh_token_cookie(user)?)
                .body("You are now logged in"))
        }
        false => {
            debug!("invalid user");
            Err(AuthError::NoCookieSigner)
        }
    }
}

#[get("/hello")]
pub async fn hello(user: User) -> impl actix_web::Responder {
    info!("hi");
    format!("Hello there, i see your user id is {}.", user.username)
}
