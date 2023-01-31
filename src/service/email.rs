use lazy_static::lazy_static;
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use log::{debug, warn};
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use tera;
use tera::{try_get_value, Context, Result, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("src/service/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

fn generate_email(username: &str, activation_link: &str) -> String {
    let mut context = Context::new();
    context.insert("username", username);
    context.insert("activation_link", activation_link);

    // A one off template
    Tera::one_off("hello", &Context::new(), true).unwrap();

    TEMPLATES.render("registration.html", &context).unwrap()
}

async fn send_registration_mail(message: String, receiver_address: &str) {
    let smtp_user = env::var("SNITCH_SMTP_USER").expect("SNITCH_SMTP_USER not defined");
    let smtp_password = env::var("SNITCH_SMTP_PASSWORD").expect("SNITCH_SMTP_PASSWORD not defined");
    let smtp_server = env::var("SNITCH_SMTP_URL").expect("SNITCH_SMTP_URL not defined");

    let email = Message::builder()
        .from("mk@quakesaver.net".parse().unwrap())
        .reply_to("noreply@snitch.cool".parse().unwrap())
        .to(receiver_address.parse().unwrap())
        .subject("Snitch User Registration")
        .body(message)
        .unwrap();

    let credentials = Credentials::new(smtp_user.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(&*smtp_server)
        .unwrap()
        .credentials(credentials)
        .authentication(vec![Mechanism::Login])
        .build();

    match mailer.send(&email) {
        Ok(_) => debug!("Email sent successfully"),
        Err(e) => warn!("Could not send email: {:?}", e),
    }
    println!("done");
}

#[tokio::test]
async fn test_email_client() {
    let test_recipient = "info@snitch.cool";
    let test_message = generate_email("Bob", "https://snitch.cool/register/isdjfolisjdflijs");
    send_registration_mail(test_message, test_recipient).await
}

#[test]
fn test_render_email() {
    generate_email("Bob", "https://snitch.cool/register/isdjfolisjdflijs");
}
