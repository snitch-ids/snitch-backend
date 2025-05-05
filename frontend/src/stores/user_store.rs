use std::fmt::{Display, Formatter};
use yewdux::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AuthenticationError {
    LoginFailed,
}

impl Display for AuthenticationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            AuthenticationError::LoginFailed => "login failed",
        };
        f.write_str(message)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Store)]
pub struct UserStore {
    pub(crate) email: Option<String>,
    pub(crate) authentication_error: Option<AuthenticationError>,
}

impl UserStore {
    pub(crate) fn is_authenticated(&self) -> bool {
        self.email.is_some() && self.authentication_error.is_none()
    }
}
