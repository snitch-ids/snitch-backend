use crate::errors::ServiceError;
use crate::model::user::UserID;
use crate::persistance::token::TokenState;
use actix_identity::Identity;
use actix_web::{delete, get, web, Responder};
use log::info;

#[get("/token/new")]
pub(crate) async fn create_token(
    id: Identity,
    token_state: web::Data<TokenState>,
) -> Result<impl Responder, ServiceError> {
    info!("generate new token request");
    let user_id: UserID = id.id().unwrap().into();
    let mut tokens = token_state.token.lock().await;
    let token = tokens.create_token_for_user_id(&user_id);
    Ok(web::Json(token))
}

#[get("/token")]
pub(crate) async fn get_token(
    id: Identity,
    token_state: web::Data<TokenState>,
) -> Result<impl Responder, ServiceError> {
    info!("get token request");
    let user_id: UserID = id.id().unwrap().into();
    let tokens = token_state.token.lock().await;
    let token = tokens.get_token_of_user_id(&user_id).unwrap();
    Ok(web::Json(token.clone()))
}
