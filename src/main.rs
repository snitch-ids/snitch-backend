mod api;
mod errors;
mod intentory;
mod model;
mod persistence;
mod service;

use crate::persistence::token::TokenState;
use actix::Actor;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::{Key, SameSite};
use std::env;
use std::str::FromStr;

use crate::api::registration::register_reply;
use actix_web::web::Data;
use actix_web::{middleware, services, web, App, HttpServer};
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

const USER_COOKIE_NAME: &str = "snitch-user";
const PORT: u16 = 8081;

use crate::api::notification_settings::get_notification_services;
use crate::api::oauth::{oauth, oauth_done};
use crate::service::notification_dispatcher::NotificationManager;
use crate::service::notification_filter::NotificationFilter;
use actix_web::http::header;

const SAME_SITE: SameSite = SameSite::Strict;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Some(filename) = std::env::args().nth(1) {
        dotenv::from_filename(filename).expect("failed parsing dotenv file");
    };

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let notification_filter = NotificationFilter::new();

    let backend_url =
        env::var("SNITCH_BACKEND_URL").expect("environment variable SNITCH_BACKEND_URL undefined");
    let frontend_url = env::var("SNITCH_FRONTEND_URL").expect("SNITCH_FRONTEND_URL undefined");
    let db_service = RedisDatabaseService::new()
        .await
        .expect("failed to create redis service");
    let notification_actor = service::notification_dispatcher::NotificationActor {
        notification_manager: NotificationManager::new(),
    };
    let notification_addr = web::Data::new(notification_actor.start());

    let db_service = RedisDatabaseService::new()
        .await
        .expect("failed to create redis service");
    let state = Data::new(AppState {
        notification_filter: Mutex::new(notification_filter),
        persist: Mutex::new(db_service),
        backend_url: Url::from_str(&backend_url)
            .unwrap_or_else(|_| panic!("failed to parse as url: {backend_url}")),
        frontend_url: Url::from_str(&frontend_url)
            .unwrap_or_else(|_| panic!("failed to parse as url: {frontend_url}")),
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
        let cors = setup_cors(&frontend_url, &backend_url);

        let services = services![
            welcome,
            add_message,
            register,
            register_reply,
            login,
            logout,
            index,
            get_messages_by_hostname,
            get_message_hostnames,
            get_user_by_id,
            delete_user,
        ];
        let services_token = services![create_token, get_token, delete_token,];

        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_http_only(true)
                .cookie_domain(cookie_domain)
                .cookie_path("/".into())
                .cookie_name(USER_COOKIE_NAME.to_string())
                .cookie_same_site(SAME_SITE)
                .cookie_secure(true)
                .build();

        App::new()
            .wrap(cors)
            .wrap(IdentityMiddleware::default())
            .wrap(session_middleware)
            .service(services)
            .service(services_token)
            .service(oauth)
            .service(oauth_done)
            .service(get_notification_services())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .app_data(notification_addr.clone())
            .app_data(state_token.clone())
    })
    .bind(("0.0.0.0", PORT))?
    .run()
    .await
}

fn setup_cors(frontend_url: &str, backend_url: &str) -> Cors {
    Cors::default()
        .allowed_origin(frontend_url)
        .allowed_origin(backend_url)
        .allowed_methods(vec!["GET", "POST", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            header::COOKIE,
            header::CONTENT_TYPE,
        ])
        .expose_headers(vec![header::SET_COOKIE])
        .supports_credentials()
        .max_age(3600)
}
