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
    users::{add_user, delete_user, get_user_by_id},
    welcome, AppStateWithCounter,
};
use log::error;

use crate::api::registration::register_reply;
use persistance::redis::RedisDatabaseService;
use serde::{Deserialize, Serialize};

use crate::api::messages::get_message_hostnames;
use tokio::sync::Mutex;

fn get_secret_key() -> Key {
    Key::generate()
}

const ONE_HOUR: Duration = Duration::minutes(60);
const USER_COOKIE_NAME: &str = "user_cookie";
const PORT: u16 = 8081;

#[cfg(not(debug_assertions))]
const COOKIE_SECURE: bool = true;
#[cfg(not(debug_assertions))]
const SAME_SITE: SameSite = SameSite::Strict;

#[cfg(debug_assertions)]
const COOKIE_SECURE: bool = false;
#[cfg(debug_assertions)]
const SAME_SITE: SameSite = SameSite::None;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Some(filename) = std::env::args().nth(1) {
        dotenv::from_filename(filename).expect("failed parsing dotenv file");
    };

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let db_service = RedisDatabaseService::new()
        .await
        .expect("failed to create redis service");

    let state = Data::new(AppStateWithCounter {
        messages: Mutex::new(db_service),
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

        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(add_message)
            .service(register)
            .service(register_reply)
            .service(login)
            .service(logout)
            .service(index)
            .service(welcome)
            .service(get_messages_by_hostname)
            .service(get_message_hostnames)
            .service(get_user_by_id)
            .service(delete_user)
            .service(create_token)
            .service(get_token)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_http_only(true)
                    .cookie_domain(cookie_domain)
                    .cookie_name(USER_COOKIE_NAME.to_string())
                    .cookie_same_site(SAME_SITE)
                    .cookie_secure(COOKIE_SECURE)
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
