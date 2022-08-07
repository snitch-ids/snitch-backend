#![feature(drain_filter)]
mod api;
mod errors;
mod intentory;
mod model;
mod persistance;
use crate::persistance::mongodb::DatabaseService;
use api::{
    messages::{add_message, get_messages_by_hostname},
    users::{add_user, delete_user, get_users},
    AppStateWithCounter,
};
use persistance::users::Users;
use tokio::sync::Mutex;

use actix_web::{web, App, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_service = DatabaseService::new("mongodb://root:kdjie234!@localhost:27017")
        .await
        .expect("failed to create monogdb service");

    let state = web::Data::new(AppStateWithCounter {
        users: Mutex::new(Users { users: vec![] }),
        messages: Mutex::new(db_service),
    });

    HttpServer::new(move || {
        App::new()
            .service(add_user)
            .service(get_users)
            .service(delete_user)
            .service(add_message)
            .service(get_messages_by_hostname)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
