use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum APIError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {_0}")]
    BadRequest(String),

    Unauthorized,
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        match self {
            APIError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error, Please try later")
            }
            APIError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            APIError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}
