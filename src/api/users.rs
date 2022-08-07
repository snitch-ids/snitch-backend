use crate::api::AppStateWithCounter;
use actix_web::{delete, get, post, web, Responder};

#[get("/users/{name}")]
async fn get_user_by_id(
    name: web::Path<String>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users
        .get_user_by_id(name.to_owned())
        .expect("failed getting user");
    format!("gotten user {added_user}")
}

#[get("/users/")]
async fn get_users(state: web::Data<AppStateWithCounter>) -> impl Responder {
    let users = state.users.lock().await;
    let added_user = users.get_users().expect("failed getting user");
    format!("gotten user {:?}", added_user)
}

#[post("/users/{name}")]
async fn add_user(
    name: web::Path<String>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let added_user = users.add_user(name.to_owned()).expect("failed adding user");
    format!("added user {added_user}")
}

#[delete("/users/{name}")]
async fn delete_user(
    name: web::Path<String>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut users = state.users.lock().await;
    let deleted_user = users
        .delete_user(name.to_owned())
        .expect("failed deleting user");
    format!("deleted user {deleted_user}")
}
