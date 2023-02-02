use crate::errors::ServiceError::InternalServerError;
use lazy_static::lazy_static;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::response::Response;
use lettre::{
    transport::smtp::authentication::Credentials, Address, Message, SmtpTransport, Transport,
};
use log::{debug, warn};
use reqwest::Url;
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use tera;
use tera::{try_get_value, Context, Result, Tera};

pub struct RegistrationMessage {
    payload: String,
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("src/service/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["*.html"]);
        tera
    };
}

pub fn generate_registration_mail(username: &str, activation_link: &Url) -> RegistrationMessage {
    let mut context = Context::new();
    context.insert("username", username);
    context.insert("activation_link", &activation_link.to_string());

    RegistrationMessage {
        payload: TEMPLATES.render("registration.html", &context).unwrap(),
    }
}

pub async fn send_registration_mail(message: RegistrationMessage, receiver: Mailbox) -> Response {
    let smtp_user = env::var("SNITCH_SMTP_USER").expect("SNITCH_SMTP_USER not defined");
    let smtp_password = env::var("SNITCH_SMTP_PASSWORD").expect("SNITCH_SMTP_PASSWORD not defined");
    let smtp_server = env::var("SNITCH_SMTP_URL").expect("SNITCH_SMTP_URL not defined");
    let email = Message::builder()
        .from("mk@quakesaver.net".parse().unwrap())
        .reply_to("noreply@snitch.cool".parse().unwrap())
        .to(receiver)
        .subject("Snitch User Registration")
        .body(message.payload)
        .unwrap();

    let credentials = Credentials::new(smtp_user.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(&*smtp_server)
        .unwrap()
        .credentials(credentials)
        .authentication(vec![Mechanism::Login])
        .build();

    mailer
        .send(&email)
        .expect("failed sending registration mail")
}

#[tokio::test]
async fn test_email_client() {
    let test_recipient = "info@snitch.cool";
    let test_message =
        generate_registration_mail("Bob", "https://snitch.cool/register/isdjfolisjdflijs");
    send_registration_mail(test_message, test_recipient.parse().unwrap())
        .await
        .unwrap();
}

#[test]
fn test_render_email() {
    generate_registration_mail(
        &"liajsdfljasdlifj.sdlfijsdlfijsdlfijsldfjdfjdf@gmail.com".to_string(),
        "https://snitch.cool/register/isdjfolisjdflijs",
    );
}
