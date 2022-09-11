use crate::api::authentication::LoginRequest;
use crate::persistance::user::User;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
const JWT_SECRET: &[u8] = b"secret";

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("jwt token creation error")]
    JWTTokenCreation,
    #[error("not authorized")]
    NoAuthorization,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub(crate) fn authenticate_user(
    user: &User,
    login_request: &LoginRequest,
) -> Result<String, AuthenticationError> {
    if user.password == login_request.password {
        let token = create_jwt(&user.email)?;
        return Ok(token);
    } else {
        Err(AuthenticationError::NoAuthorization)
    }
}

pub fn create_jwt(email: &str) -> Result<String, AuthenticationError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: email.to_owned(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| AuthenticationError::JWTTokenCreation)
}
