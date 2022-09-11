use crate::{
    api::{authentication::LoginRequest, AppStateWithCounter},
    authentication_service,
    persistance::{
        user::{self, User},
        Persist,
    },
};
use actix_web::{delete, get, post, web, Responder};
use anyhow::Result;

#[get("/users/")]
async fn get_users(state: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut state = state.messages.lock().await;
    let users = state.get_users().await.unwrap();
    let payload = serde_json::to_string(&users).unwrap();
    println!("send {:?}", payload);
    payload
}

#[post("/users/")]
async fn add_user(user: web::Json<User>, state: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut state = state.messages.lock().await;
    let added_user = state.add_user(&user).await.expect("failed adding user");
    format!("added user {:?}", added_user)
}

pub fn asdf(email: String) -> Result<User> {
    Ok(User::default())
}

#[post("/users/authenticate")]
async fn authenticate_user(
    login_request: web::Json<LoginRequest>,
    state: web::Data<AppStateWithCounter>,
) -> impl Responder {
    let mut state = state.messages.lock().await;
    let user = state
        .get_user_by_email(&login_request.email)
        .await
        .expect("failed getting user by email");
    authentication_service::authenticate_user(&user, &login_request);
    format!("authenticated")
}
