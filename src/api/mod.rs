pub mod authentication;
pub mod messages;
pub mod registration;
pub mod token;
pub mod users;

use actix_web::{get, Responder};
use log::debug;
use reqwest::Url;
use tokio::sync::Mutex;

use crate::persistence::redis::RedisDatabaseService;

pub struct AppState {
    pub messages: Mutex<RedisDatabaseService>,
    pub backend_url: Url,
    pub frontend_url: Url,
}

#[get("/")]
pub(crate) async fn welcome() -> impl Responder {
    debug!("welcome request");
    "welcome"
}
