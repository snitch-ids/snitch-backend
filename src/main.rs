#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;

use actix_cors::Cors;
use actix_jwt_auth_middleware::{AuthError, UseJWTOnScope};
use actix_jwt_auth_middleware::{
    AuthResult, AuthenticationService, Authority, CookieSigner, FromRequest,
};
use jwt_compact::alg::Ed25519;

use crate::api::users::get_user_by_id;
use crate::persistance::users::User;
use actix_web::web::Data;
use actix_web::{get, http, post, web, App, HttpResponse, HttpServer};
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, delete_user, get_users},
    welcome, AppStateWithCounter,
};
use exonum_crypto::KeyPair;
use log::{debug, info};
use persistance::{redis::RedisDatabaseService, users::Users};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // let db_service = MongoDatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
    //     .await
    //     .expect("failed to create monogdb service");

    let db_service = RedisDatabaseService::new("redis://127.0.0.1:6379")
        .await
        .expect("failed to create redis service");

    let state = Data::new(AppStateWithCounter {
        users: Mutex::new(Users::example()),
        messages: Mutex::new(db_service),
    });

    let port = 8080;
    println!("starting server on port {port}");

    let key_pair = KeyPair::random();

    let cookie_signer = CookieSigner::new()
        .signing_key(key_pair.secret_key().clone())
        .algorithm(Ed25519)
        .build()
        .expect("failed cookie signer");

    let authority = Authority::<User, _, _, _>::new()
        .refresh_authorizer(|| async move { Ok(()) })
        .cookie_signer(Some(cookie_signer.clone()))
        .verifying_key(key_pair.public_key())
        .build()
        .expect("failed authority");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&format!("http://127.0.0.1:{port}"))
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".rust-lang.org"))
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(9999999999);

        App::new()
            .wrap(cors)
            .service(welcome)
            .service(login)
            .app_data(Data::new(cookie_signer.clone()))
            .app_data(state.clone())
            .service(
                web::scope("")
                    .service(hello)
                    .service(add_user)
                    .service(get_user_by_id)
                    .service(get_users)
                    .service(delete_user)
                    .service(add_message)
                    .service(get_messages_by_hostname)
                    .use_jwt(authority.clone()),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

#[derive(Deserialize, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(
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
                .cookie(cookie_signer.create_access_token_cookie(&user)?)
                .cookie(cookie_signer.create_refresh_token_cookie(&user)?)
                .body("You are now logged in"))
        }
        false => {
            debug!("invalid user");
            Err(AuthError::NoCookieSigner)
        }
    }
}

#[get("/hello")]
async fn hello(user: User) -> impl actix_web::Responder {
    info!("hi");
    format!("Hello there, i see your user id is {}.", user.username)
}
