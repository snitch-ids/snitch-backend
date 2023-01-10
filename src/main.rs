#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;
mod service;

use actix_cors::Cors;
use actix_jwt_auth_middleware::{Authority, CookieSigner, UseJWTOnScope};
use jwt_compact::alg::Ed25519;

use crate::api::authentication::{hello, login};
use crate::api::users::get_token;
use crate::model::user::User;
use crate::persistance::token::{TokenState, TokenStore};
use actix_web::web::Data;
use actix_web::{get, http, web, App, HttpServer};
use actix_web_httpauth::extractors::bearer::{self, BearerAuth, Config};
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, create_token, delete_user, get_user_by_id, get_users},
    welcome, AppStateWithCounter,
};
use exonum_crypto::KeyPair;
use log::info;
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

    let state_token = Data::new(TokenState::new());

    let port = 8081;
    let url_frontend = "http://127.0.0.1:8080".to_string();
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
            .allowed_origin(&url_frontend)
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".rust-lang.org"))
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(9999999999);

        App::new()
            .wrap(cors)
            .service(welcome)
            .service(add_message)
            .service(login)
            .app_data(Data::new(cookie_signer.clone()))
            .app_data(state.clone())
            .app_data(state_token.clone())
            .app_data(
                Config::default()
                    .realm("Restricted area")
                    .scope("email photo"),
            )
            .service(
                web::scope("")
                    .service(hello)
                    .service(add_user)
                    .service(get_user_by_id)
                    .service(get_users)
                    .service(delete_user)
                    .service(get_messages_by_hostname)
                    .service(create_token)
                    .service(get_token)
                    .use_jwt(authority.clone()),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
