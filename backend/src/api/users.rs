use crate::api::AppState;
use crate::errors::APIError;
use crate::model::user::{User, UserID};
use crate::service::authentication::hash_password;
use actix_identity::Identity;
use actix_web::{delete, get, post, web, Responder};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserResponse {
    pub(crate) email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse { email: user.email }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddUserRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[get("/user")]
pub async fn get_user_by_id(
    id: Identity,
    state: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    let user_id: UserID = id.id().unwrap().into();
    let mut users = state.persist.lock().await;
    let user = users.get_user_by_id(&user_id).await;
    let response = UserResponse::from(user);
    Ok(web::Json(response))
}

#[post("/user")]
pub(crate) async fn add_user(
    state: web::Data<AppState>,
    user: web::Json<AddUserRequest>,
) -> Result<impl Responder, APIError> {
    info!("add user");

    let new_user = User::new(user.email.clone(), hash_password(&user.password));

    let mut users = state.persist.lock().await;
    let added_user = users.add_user(&new_user).await;
    Ok(web::Json(added_user))
}

#[delete("/user")]
pub(crate) async fn delete_user(
    id: Identity,
    state: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    let mut users = state.persist.lock().await;
    let user_id: UserID = id.id().unwrap().into();
    let deleted_user = users.delete_user(&user_id).await;
    Ok(web::Json(deleted_user))
}

#[cfg(test)]
mod tests {
    use crate::api::users::UserResponse;
    use crate::model::user::User;

    #[test]
    fn test_user_response() {
        let _ = UserResponse::from(User::example());
    }
}
