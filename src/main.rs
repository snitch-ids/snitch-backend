#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, delete_user, get_users},
    welcome, AppStateWithCounter,
};
use persistance::{redis::RedisDatabaseService, users::Users};
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

    let state = web::Data::new(AppStateWithCounter {
        users: Mutex::new(Users::example()),
        messages: Mutex::new(db_service),
    });

    println!("starting server...");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".rust-lang.org"))
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(welcome)
            .service(add_user)
            .service(get_users)
            .service(delete_user)
            .service(add_message)
            .service(get_messages_by_hostname)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await
}
