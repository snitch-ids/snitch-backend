pub mod authentication;
pub mod messages;
pub(crate) mod notification_settings;
pub mod oauth;
pub mod registration;
pub mod token;
pub mod users;

use actix_web::{get, Responder};
use log::debug;
use openidconnect::{CsrfToken, Nonce};
use reqwest::Url;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::persistence::redis::RedisDatabaseService;
use crate::service::notification_filter::NotificationFilter;

pub struct AppState {
    pub persist: Mutex<RedisDatabaseService>,
    pub backend_url: Url,
    pub frontend_url: Url,
    pub csrf_token: Mutex<HashMap<String, String>>,
    pub(crate) notification_filter: Mutex<NotificationFilter>,
}

#[get("/")]
pub(crate) async fn welcome() -> impl Responder {
    debug!("welcome request");
    "welcome"
}
