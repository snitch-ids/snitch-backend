use crate::api::AppStateWithCounter;
use crate::errors::ServiceError;
use crate::errors::ServiceError::InternalServerError;
use crate::model::user::{User, UserID};
use crate::service::authentication::hash_password;
use crate::{Deserialize, Serialize, TokenState};
use actix_identity::Identity;
use actix_web::{delete, get, post, web, Responder};
use log::info;

#[get("/user")]
pub async fn get_user_by_id(
    id: Identity,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let user_id: UserID = id.id().unwrap().into();
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(user_id)
        .map_err(|_e| InternalServerError)?;

    Ok(web::Json(added_user))
}

#[get("/user/all")]
pub(crate) async fn get_users(
    _id: Identity,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    info!("request users ... needs check for admin rights");
    //if is_admin(id) {
    let users = state.users.lock().await;
    let added_user = users.get_users().map_err(|_e| InternalServerError)?;
    Ok(web::Json(added_user))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    let new_user = User::new(user.username.clone(), hash_password(&user.password));

    let mut users = state.users.lock().await;
    let added_user = users.add_user(new_user).map_err(|_e| InternalServerError)?;
    Ok(web::Json(added_user))
}

#[delete("/user")]
pub(crate) async fn delete_user(
    id: Identity,
    state: web::Data<AppStateWithCounter>,
) -> Result<impl Responder, ServiceError> {
    let mut users = state.users.lock().await;
    let user_id: UserID = id.id().unwrap().into();
    let deleted_user = users
        .delete_user(user_id)
        .map_err(|_e| InternalServerError)?;
    Ok(web::Json(deleted_user))
}
