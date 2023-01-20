#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;
mod service;

use actix_cors::Cors;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::storage::RedisSessionStore;

use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};

use crate::api::authentication::{index, login, logout};
use crate::api::users::get_token;
use crate::errors::ServiceError;
use crate::model::user::User;
use crate::persistance::token::{TokenState, TokenStore};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{
    error, get, http, middleware, web, App, HttpMessage, HttpResponse, HttpServer, Responder,
};
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, create_token, delete_user, get_user_by_id, get_users},
    welcome, AppStateWithCounter,
};
use log::info;
use persistance::{redis::RedisDatabaseService, users::Users};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

fn get_secret_key() -> Key {
    Key::generate()
}

const ONE_MINUTE: Duration = Duration::minutes(1);
const USER_COOKIE_NAME: &str = "user_cookie";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // let db_service = MongoDatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
    //     .await
    //     .expect("failed to create monogdb service");

    let db_service = RedisDatabaseService::new("redis://localhost:6379")
        .await
        .expect("failed to create redis service");

    let state = Data::new(AppStateWithCounter {
        users: Mutex::new(Users::example()),
        messages: Mutex::new(db_service),
    });

    let state_token = Data::new(TokenState::new());

    let port = 8081;
    println!("starting server on port {port}");

    let secret_key = get_secret_key();

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(login)
            .service(logout)
            .service(index)
            .service(add_message)
            .service(welcome)
            .service(get_messages_by_hostname) // for testing no auth
            .service(add_user)
            .service(get_user_by_id)
            .service(get_users)
            .service(delete_user)
            .service(create_token)
            .service(get_token)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name(USER_COOKIE_NAME.to_string())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(ONE_MINUTE))
                    .build(),
            )
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .app_data(state_token.clone())
    })
    .bind(("localhost", port))?
    .run()
    .await
}
