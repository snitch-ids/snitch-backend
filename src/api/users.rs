use crate::persistance::users::User;
use crate::{api::AppStateWithCounter, Users};
use actix_web::{delete, get, post, web, Responder};

#[get("/users/{user_id}")]
async fn get_user_by_id(
    user_id: web::Path<i64>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(user_id.into_inner())
        .expect("failed getting user");
    format!("gotten user {added_user}")
}

#[get("/users/")]
pub(crate) async fn get_users(state: web::Data<AppStateWithCounter>) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users.get_users().expect("failed getting user");
    format!("gotten user {:?}", added_user)
}

#[post("/users/")]
pub(crate) async fn add_user(
    state: web::Data<AppStateWithCounter>,
    user: web::Json<User>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let added_user = users
        .add_user(user.into_inner())
        .expect("failed adding user");
    format!("added user {added_user}")
}

#[delete("/users/{user_id}")]
pub(crate) async fn delete_user(
    user_id: web::Path<i64>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let deleted_user = users
        .delete_user(user_id.into_inner())
        .expect("failed deleting user");
    format!("deleted user {deleted_user}")
}
