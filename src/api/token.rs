use crate::errors::APIError;
use crate::model::user::UserID;
use crate::persistance::token::TokenState;
use actix_identity::Identity;
use actix_web::{get, web, Responder};
use log::info;
use std::collections::HashSet;

#[get("/token/new")]
pub(crate) async fn create_token(
    id: Identity,
    token_state: web::Data<TokenState>,
) -> Result<impl Responder, APIError> {
    info!("generate new token request");
    let user_id: UserID = id.id().unwrap().into();
    let mut tokens = token_state.token.lock().await;
    let token = tokens.create_token_for_user_id(&user_id).await;
    Ok(web::Json(token))
}

#[get("/token")]
pub(crate) async fn get_token(
    id: Identity,
    token_state: web::Data<TokenState>,
) -> Result<impl Responder, APIError> {
    info!("get token request");
    let user_id: UserID = id.id().unwrap().into();
    let mut tokens = token_state.token.lock().await;
    if let Some(token) = tokens.get_token_of_user_id(&user_id).await {
        Ok(web::Json(token.clone()))
    } else {
        Ok(web::Json(vec![]))
    }
}
