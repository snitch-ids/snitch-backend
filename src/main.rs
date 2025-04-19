mod api;
mod errors;
mod intentory;
mod model;
mod persistence;
mod service;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use std::env;
use std::str::FromStr;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};

use crate::persistence::token::TokenState;
use actix_web::cookie::{Key, SameSite};

use crate::api::registration::register_reply;
use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use api::{
    authentication::{index, login, logout},
    messages::{add_message, get_messages_by_hostname},
    registration::register,
    token::{create_token, get_token},
    users::{delete_user, get_user_by_id},
    welcome, AppState,
};
use log::error;
use persistence::redis::RedisDatabaseService;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::api::messages::get_message_hostnames;
use crate::api::token::delete_token;
use tokio::sync::Mutex;

fn get_secret_key() -> Key {
    Key::generate()
}

const USER_COOKIE_NAME: &str = "user_cookie";
const PORT: u16 = 8081;

#[cfg(not(debug_assertions))]
const SAME_SITE: SameSite = SameSite::Strict;

#[cfg(debug_assertions)]
const SAME_SITE: SameSite = SameSite::Lax;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Some(filename) = std::env::args().nth(1) {
        dotenv::from_filename(filename).expect("failed parsing dotenv file");
    };

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let db_service = RedisDatabaseService::new()
        .await
        .expect("failed to create redis service");

    let backend_url =
        env::var("SNITCH_BACKEND_URL").expect("environment variable SNITCH_BACKEND_URL undefined");
    let reply_url = env::var("SNITCH_FRONTEND_URL").expect("SNITCH_FRONTEND_URL undefined");

    let state = Data::new(AppState {
        messages: Mutex::new(db_service),
        backend_url: Url::from_str(&backend_url)
            .unwrap_or_else(|_| panic!("failed to parse as url: {backend_url}")),
        reply_url: Url::from_str(&reply_url)
            .unwrap_or_else(|_| panic!("failed to parse as url: {reply_url}")),
    });

    let db_token_service = RedisDatabaseService::new()
        .await
        .expect("failed to create redis service");

    let state_token = Data::new(TokenState::new(db_token_service.connection));
    let secret_key = get_secret_key();

    HttpServer::new(move || {
        let cookie_domain = std::env::var("SNITCH_COOKIE_DOMAIN")
            .map_err(|e| {
                error!("Failed loading SNITCH_COOKIE_DOMAIN: {e}");
                std::process::exit(1)
            })
            .ok();
        let cors = setup_cors();

        App::new()
            .wrap(cors)
            .service(welcome)
            .service(add_message)
            .service(register)
            .service(register_reply)
            .service(login)
            .service(logout)
            .service(index)
            .service(get_messages_by_hostname)
            .service(get_message_hostnames)
            .service(get_user_by_id)
            .service(delete_user)
            .service(create_token)
            .service(get_token)
            .service(delete_token)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_http_only(true)
                    .cookie_domain(cookie_domain)
                    .cookie_name(USER_COOKIE_NAME.to_string())
                    .cookie_same_site(SAME_SITE)
                    .cookie_secure(true)
                    .build(),
            )
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .app_data(state_token.clone())
    })
    .bind(("0.0.0.0", PORT))?
    .run()
    .await
}

fn setup_cors() -> Cors {
    #[cfg(debug_assertions)]
    let cors = Cors::permissive();
    #[cfg(not(debug_assertions))]
    let cors = Cors::default()
        .allowed_origins(vec!["https://api.snitch.cool", "https://snitch.cool"])
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .max_age(3600);
    cors
}
