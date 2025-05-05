use crate::pages::login::LoginRequest;
use crate::pages::register::RegisterRequest;
use crate::stores::user_store::{AuthenticationError, UserStore};

use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use wasm_cookies::cookies::get;
use web_sys::console::{log_1, warn_1};
use web_sys::RequestCredentials;

use yewdux::prelude::*;

const BACKEND_URL: &str = env!("SNITCH_BACKEND_URL"); // See Dockerfile
const USER_COOKIE_NAME: &str = "snitch_user";
pub type MessageToken = String;

const INCLUDE_CREDENTIALS: RequestCredentials = RequestCredentials::Include;

#[derive(Serialize)]
pub struct MessagesRequest<'a> {
    pub(crate) hostname: &'a str,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserResponse {
    pub(crate) email: String,
}

pub fn register_user(register_request: &RegisterRequest) {
    let request = register_request.clone();

    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("{BACKEND_URL}/register");

        let msg = format!("calling url {url}");
        log_1(&msg.into());

        let payload = to_string(&request).unwrap();
        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "*/*")
            .body(payload)
            .send()
            .await
            .unwrap();
        let msg = format!("auth status: {}", response.status());
        log_1(&msg.into());
    });
}

async fn get_user_info() -> Result<UserResponse, FetchError> {
    let url = format!("{BACKEND_URL}/user");
    let msg = format!("calling url {url}");
    log_1(&msg.to_string().into());

    let response = Request::get(&url)
        .credentials(INCLUDE_CREDENTIALS)
        .send()
        .await
        .map_err(|_e| FetchError::UserInfo)?;

    let msg = format!("auth status: {}", response.status());
    log_1(&msg.into());

    response
        .json::<UserResponse>()
        .await
        .map_err(|_e| FetchError::NoMessage)
}

pub fn authenticate(login_request: LoginRequest, dispatch: Dispatch<UserStore>) {
    wasm_bindgen_futures::spawn_local(async move {
        log_1(&"calling url".to_string().into());
        let login_url = format!("{BACKEND_URL}/login");
        let payload = to_string(&login_request).unwrap();
        let (email, authentication_error) = Request::post(&login_url)
            .header("Content-Type", "application/json")
            .header("Accept", "*/*")
            .credentials(INCLUDE_CREDENTIALS)
            .body(&payload)
            .send()
            .await
            .map_or_else(
                |e| {
                    warn_1(&e.to_string().into());
                    (None, Some(AuthenticationError::LoginFailed))
                },
                |response| match response.status() {
                    200 => (Some(login_request.email), None),
                    _ => (None, Some(AuthenticationError::LoginFailed)),
                },
            );

        dispatch.set(UserStore {
            email,
            authentication_error,
        });
    });
}

pub fn logout(dispatch: Dispatch<UserStore>) {
    wasm_bindgen_futures::spawn_local(async move {
        log_1(&"calling url".to_string().into());
        let url = format!("{BACKEND_URL}/logout");
        let response = Request::post(&url)
            .header("Accept", "*/*")
            .credentials(INCLUDE_CREDENTIALS)
            .send()
            .await
            .unwrap();
        let msg = format!("logout status: {}", response.status());
        dispatch.set(UserStore {
            email: None,
            ..Default::default()
        });
        log_1(&msg.into());
    });
}

pub fn authenticated() -> bool {
    get(USER_COOKIE_NAME, USER_COOKIE_NAME).is_some()
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct MessageBackend {
    pub hostname: String,
    pub title: String,
    pub body: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub enum FetchError {
    NoMessage,
    UserInfo,
}

pub async fn request_hostnames() -> Result<Vec<String>, FetchError> {
    let msg = "requesting hostnames".to_string();
    log_1(&msg.into());

    let messages_url = format!("{BACKEND_URL}/hostnames");
    let response = Request::get(&messages_url)
        .header("Content-Type", "application/json")
        .credentials(INCLUDE_CREDENTIALS)
        .send()
        .await
        .unwrap();

    let msg = format!("requesting done {:?}", response.status());
    log_1(&msg.into());

    response
        .json::<Vec<String>>()
        .await
        .map_err(|_e| FetchError::NoMessage)
}

pub async fn request_messages(hostname: &str) -> Result<Vec<MessageBackend>, FetchError> {
    let msg = format!("requesting data for hostname {}", hostname);
    log_1(&msg.into());
    let messages_url = format!("{BACKEND_URL}/messages/{hostname}");
    let response = Request::get(&messages_url)
        .credentials(INCLUDE_CREDENTIALS)
        .send()
        .await
        .unwrap();

    response
        .json::<Vec<MessageBackend>>()
        .await
        .map_err(|_e| FetchError::NoMessage)
}

pub async fn create_token() -> MessageToken {
    log_1(&"want token".into());
    let messages_url = format!("{BACKEND_URL}/token");
    let response = Request::post(&messages_url)
        .credentials(INCLUDE_CREDENTIALS)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();

    response
        .json::<MessageToken>()
        .await
        .map_err(|_e| FetchError::NoMessage)
        .expect("TODO: panic token")
}

pub async fn request_tokens() -> Result<Vec<MessageToken>, FetchError> {
    log_1(&"want token".into());
    let messages_url = format!("{BACKEND_URL}/token");
    let response = Request::get(&messages_url)
        .credentials(INCLUDE_CREDENTIALS)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();
    response
        .json::<Vec<MessageToken>>()
        .await
        .map_err(|_e| FetchError::NoMessage)
}

pub async fn revoke_token(token: &str) -> u16 {
    log_1(&"revoke token".into());
    let messages_url = format!("{BACKEND_URL}/token/{token}");
    let response = Request::delete(&messages_url)
        .credentials(INCLUDE_CREDENTIALS)
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();
    response.status()
}

pub fn test() {
    wasm_bindgen_futures::spawn_local(async move {
        let messages_url = format!("{BACKEND_URL}/");
        let msg = format!("requesting data for hostname {}", messages_url);
        log_1(&msg.into());
        let request = Request::get(&messages_url).credentials(INCLUDE_CREDENTIALS);
        request.send().await.unwrap();
    })
}
