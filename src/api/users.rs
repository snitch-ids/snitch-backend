use crate::api::AppStateWithCounter;
use crate::errors::ServiceError;
use crate::errors::ServiceError::InternalServerError;
use crate::model::user::{User, UserID};
use crate::service::authentication::hash_password;
use crate::{Deserialize, Serialize, TokenState};
use actix_jwt_auth_middleware::FromRequest;
use actix_web::{delete, get, post, web, Responder};
use log::info;

#[get("/user/{user_id}")]
pub async fn get_user_by_id(
    user_id: web::Path<UserID>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(user_id.into_inner())
        .map_err(|e| InternalServerError)?;

    Ok(web::Json(added_user))
}

#[get("/user")]
pub(crate) async fn get_users(
    _user: User,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    info!("request users");
    let users = state.users.lock().await;
    let added_user = users.get_users().map_err(|e| InternalServerError)?;
    Ok(web::Json(added_user))
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRequest)]
pub struct AddUserRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[post("/user")]
pub(crate) async fn add_user(
    state: web::Data<AppStateWithCounter>,
    user: web::Json<AddUserRequest>,
    // This should be a proper actix error. Test this.
) -> Result<impl Responder, ServiceError> {
    info!("add user");

    let new_user = User {
        username: user.username.clone(),
        password_hash: hash_password(&user.password),
    };

    let mut users = state.users.lock().await;
    let added_user = users.add_user(new_user).map_err(|e| InternalServerError)?;
    Ok(web::Json(added_user))
}

#[delete("/user/{user_id}")]
pub(crate) async fn delete_user(
    user_id: web::Path<UserID>,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let mut users = state.users.lock().await;
    let deleted_user = users
        .delete_user(user_id.into_inner())
        .map_err(|e| InternalServerError)?;
    Ok(web::Json(deleted_user))
}

#[get("/user/{user_id}/token/new")]
pub(crate) async fn create_token(
    user_id: web::Path<UserID>,
    token_state: web::Data<TokenState>,
) -> Result<impl Responder, ServiceError> {
    info!("generate new token request");
    let mut tokens = token_state.token.lock().await;
    let token = tokens.create_token_for_user_id(&user_id.into_inner());
    Ok(web::Json(token))
}

#[get("/user/{user_id}/token/")]
pub(crate) async fn get_token(
    user_id: web::Path<UserID>,
    token_state: web::Data<TokenState>,
) -> impl Responder {
    info!("get token request");
    let tokens = token_state.token.lock().await;
    let token = tokens.get_token_of_user_id(&user_id.into_inner());
    format!("{token:?}")
}
