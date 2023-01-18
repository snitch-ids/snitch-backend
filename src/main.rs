#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;
mod service;

use actix_cors::Cors;
use actix_session::storage::RedisSessionStore;

use actix_jwt_auth_middleware::{Authority, CookieSigner, UseJWTOnScope};
use actix_session::{Session, SessionMiddleware};
use jwt_compact::alg::Ed25519;

use crate::api::authentication::{hello, login};
use crate::api::users::get_token;
use crate::errors::ServiceError;
use crate::model::user::User;
use crate::persistance::token::{TokenState, TokenStore};
use actix_web::cookie::Key;
use actix_web::web::Data;
use actix_web::{get, http, web, App, HttpResponse, HttpServer, Responder};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
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
    let redis_connection_string = "redis://localhost:6379";
    let store = RedisSessionStore::new(redis_connection_string)
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(state_token.clone())
            .service(add_message)
            .service(login)
            .service(welcome)
            .service(hello)
            .service(get_messages_by_hostname) // for testing no auth
            .service(add_user)
            .service(get_user_by_id)
            .service(get_users)
            .service(delete_user)
            .service(create_token)
            .service(get_token)
    })
    .bind(("localhost", port))?
    .run()
    .await
}
