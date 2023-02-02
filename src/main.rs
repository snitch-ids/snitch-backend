#![feature(drain_filter)]
extern crate core;

mod api;
mod errors;
mod intentory;
mod model;
mod persistance;
mod service;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};

use crate::persistance::token::TokenState;
use actix_web::cookie::time::Duration;
use actix_web::cookie::{Key, SameSite};

use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use api::{
    authentication::{index, login, logout},
    messages::{add_message, get_messages_by_hostname},
    registration::register,
    token::{create_token, get_token},
    users::{add_user, delete_user, get_user_by_id, get_users},
    welcome, AppStateWithCounter,
};

use crate::api::registration::register_reply;
use persistance::{redis::RedisDatabaseService, users::Users};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

fn get_secret_key() -> Key {
    Key::generate()
}

const ONE_HOUR: Duration = Duration::minutes(60);
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
            .service(register)
            .service(register_reply)
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
                    .cookie_same_site(SameSite::None)
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::default().session_ttl(ONE_HOUR))
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
