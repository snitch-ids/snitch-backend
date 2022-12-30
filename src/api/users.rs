use crate::api::AppStateWithCounter;
use crate::persistance::users::User;
use actix_web::{delete, get, post, web, Responder};
use log::info;

#[get("/user/{user_id}")]
pub async fn get_user_by_id(
    user_id: web::Path<i64>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(user_id.into_inner())
        .expect("failed getting user");
    format!("gotten user {added_user}")
}


// #[get("/hello")]
// async fn hello(user: User) -> impl actix_web::Responder {
//     info!("hi");
//     format!("Hello there, i see your user id is {}.", user.id)
// }


#[get("/user/")]
pub(crate) async fn get_users(user: User, state: web::Data<AppStateWithCounter>) -> impl Responder {
    info!("request users");
    let users = state.users.lock().await;
    let added_user = users.get_users().expect("failed getting user");
    format!("gotten user {added_user:?}")
}

#[post("/user/")]
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
    user_id: web::Path<i64>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let deleted_user = users
        .delete_user(user_id.into_inner())
        .expect("failed deleting user");
    format!("deleted user {deleted_user}")
}
