use crate::api::AppStateWithCounter;
use crate::model::user::{User, UserID};
use crate::TokenState;
use actix_web::{delete, get, post, web, Responder};
use log::info;

#[get("/user/{user_id}")]
pub async fn get_user_by_id(
    user_id: web::Path<UserID>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(user_id.into_inner())
        .expect("failed getting user");
    format!("gotten user {added_user}")
}

#[get("/user")]
pub(crate) async fn get_users(
    _user: User,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    info!("request users");
    let users = state.users.lock().await;
    let added_user = users.get_users().expect("failed getting user");
    format!("gotten user {added_user:?}")
}

#[post("/user")]
pub(crate) async fn add_user(
    state: web::Data<AppStateWithCounter>,
    user: web::Json<User>,
) -> impl Responder {
    info!("add user");
    let mut users = state.users.lock().await;
    let added_user = users
        .add_user(user.into_inner())
        .expect("failed adding user");
    format!("added user {added_user}")
}

#[delete("/user/{user_id}")]
pub(crate) async fn delete_user(
    user_id: web::Path<UserID>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let deleted_user = users
        .delete_user(user_id.into_inner())
        .expect("failed deleting user");
    format!("deleted user {deleted_user}")
}

#[get("/user/{user_id}/token/new")]
pub(crate) async fn create_token(
    user_id: web::Path<UserID>,
    token_state: web::Data<TokenState>,
) -> impl Responder {
    info!("generate new token request");
    let mut tokens = token_state.token.lock().await;
    let token = tokens.create_token_for_user_id(&user_id.into_inner());
    format!("{token}")
}

#[get("/user/{user_id}/token/")]
pub(crate) async fn get_token(
    user_id: web::Path<UserID>,
    token_state: web::Data<TokenState>,
) -> impl Responder {
    info!("get token request");
    let tokens = token_state.token.lock().await;
    let token = tokens.get_token_of_user_id(&user_id.into_inner());
    format!("found tokens for user {token:?}")
}
