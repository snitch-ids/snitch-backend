pub mod authentication;
pub mod messages;
pub mod registration;
pub mod token;
pub mod users;

use actix_web::{get, Responder};
use log::debug;
use tokio::sync::Mutex;

use crate::persistance::{redis::RedisDatabaseService, users::Users};

pub struct AppStateWithCounter {
    pub users: Mutex<Users>,
    pub messages: Mutex<RedisDatabaseService>,
}

#[get("/")]
pub(crate) async fn welcome() -> impl Responder {
    debug!("welcome request");
    "welcome".to_string()
}
