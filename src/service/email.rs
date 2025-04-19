use lazy_static::lazy_static;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Mechanism;

use lettre::transport::smtp::response::Response;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use reqwest::Url;

use lettre::transport::smtp::Error;
use std::env;

use tera;
use tera::{Context, Tera};

pub struct RegistrationMessage {
    payload: String,
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("src/service/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {e}");
                ::std::process::exit(1);
            }
        };

        let raw_template = include_str!("templates/registration.html");
        tera.add_raw_template("registration.html", raw_template)
            .expect("failed adding template");
        tera.autoescape_on(vec!["*.html"]);
        tera
    };
}

pub fn generate_registration_mail(email: &str, activation_link: &Url) -> RegistrationMessage {
    let mut context = Context::new();
    context.insert("email", email);
    context.insert("activation_link", &activation_link.to_string());

    RegistrationMessage {
        payload: TEMPLATES.render("registration.html", &context).unwrap(),
    }
}

pub async fn send_registration_mail(
    message: RegistrationMessage,
    receiver: Mailbox,
) -> Result<Response, Error> {
    let smtp_user = env::var("SNITCH_SMTP_USER").expect("SNITCH_SMTP_USER not defined");
    let smtp_password = env::var("SNITCH_SMTP_PASSWORD").expect("SNITCH_SMTP_PASSWORD not defined");
    let smtp_server = env::var("SNITCH_SMTP_URL").expect("SNITCH_SMTP_URL not defined");
    let port = 465;

    let email = Message::builder()
        .from("noreply@snitch.cool".parse().unwrap())
        .reply_to("noreply@snitch.cool".parse().unwrap())
        .to(receiver)
        .subject("Snitch User Registration")
        .body(message.payload)
        .unwrap();

    let credentials = Credentials::new(smtp_user, smtp_password);
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(credentials)
        .port(port)
        .authentication(vec![Mechanism::Login])
        .build();

    mailer.send(&email)
}

#[tokio::test]
async fn test_email_client() {
    let test_recipient = "marius.kriegerowski@gmail.com";
    let test_message = generate_registration_mail(
        "Bob",
        &Url::parse("https://snitch.cool/register/isdjfolisjdflijs").unwrap(),
    );
    assert!(
        send_registration_mail(test_message, test_recipient.parse().unwrap())
            .await
            .is_ok()
    );
}

#[test]
fn test_render_email() {
    generate_registration_mail(
        "liajsdfljasdlifj.sdlfijsdlfijsdlfijsldfjdfjdf@gmail.com",
        &Url::parse("https://snitch.cool/register/isdjfolisjdflijs").unwrap(),
    );
}
