#![feature(drain_filter)]
mod api;
mod authentication_service;
mod errors;
mod intentory;
mod model;
mod persistance;

use actix_cors::Cors;
use actix_web::service::ServiceRequest;
use actix_web::{http, web, App, HttpServer};
use actix_web::{middleware, Error};
use actix_web_httpauth::extractors::bearer::Config;
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth, AuthenticationError};
use actix_web_httpauth::extractors::{
    bearer::{self},
    AuthExtractorConfig,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, get_users},
    AppStateWithCounter,
};
use log::info;
use persistance::{redis::RedisDatabaseService, user::User};
use tokio::sync::Mutex;

use crate::api::users::authenticate_user;

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // provide a list of paths here that should be accessible.
    if req.path() == "/register/" {
        return Ok(req);
    }

    if credentials.token() == "affe" {
        Ok(req)
    } else {
        let config = req
            .app_data::<bearer::Config>()
            .cloned()
            .unwrap_or_default()
            .scope("urn:example:channel=HBO&urn:example:rating=G,PG-13");
        Err((AuthenticationError::from(config).into(), req))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // let db_service = MongoDatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
    //     .await
    //     .expect("failed to create monogdb service");

    let db_service = RedisDatabaseService::new("redis://127.0.0.1:6379")
        .await
        .expect("failed to create redis service");

    let state = web::Data::new(AppStateWithCounter {
        messages: Mutex::new(db_service),
    });

    println!("starting server...");
    HttpServer::new(move || {
        let middleware = HttpAuthentication::bearer(validator);

        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".rust-lang.org"))
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware)
            .wrap(middleware::Logger::default())
            .service(add_user)
            .service(get_users)
            .service(authenticate_user)
            .service(add_message)
            .service(get_messages_by_hostname)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 8082))?
    .workers(1)
    .run()
    .await
}
