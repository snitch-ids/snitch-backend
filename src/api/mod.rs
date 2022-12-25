pub mod messages;
pub mod users;

use actix_web::{get, Responder};
use tokio::sync::Mutex;

use crate::persistance::{redis::RedisDatabaseService, users::Users};

pub struct AppStateWithCounter {
    pub users: Mutex<Users>,
    pub messages: Mutex<RedisDatabaseService>,
}

#[get("/")]
pub(crate) async fn welcome() -> impl Responder {
    "welcome".to_string()
}
