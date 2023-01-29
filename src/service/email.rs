use lettre::transport::smtp::authentication::Credentials;
use lettre::Message as LettreMessage;
use lettre::{SmtpTransport, Transport};
use log::{debug, error};
use std::env;

async fn send_registration_mail(message: String) {
    let smtp_user = env::var("SNITCH_SMTP_USER").expect("SNITCH_SMTP_USER not defined");
    let smtp_password = env::var("SNITCH_SMTP_PASSWORD").expect("SNITCH_SMTP_PASSWORD not defined");
    let smtp_server = env::var("SNITCH_SMTP_SERVER").expect("SNITCH_SMTP_SERVER not defined");
    let receiver_address =
        env::var("SNITCH_RECEIVER_ADDRESS").expect("SNITCH_RECEIVER_ADDRESS not defined");

    let email = LettreMessage::builder()
        .from("Snitch <noreply@snitch.cool>".parse().unwrap())
        .reply_to("noreply@snitch.cool".parse().unwrap())
        .to(receiver_address.parse().unwrap())
        .subject("Snitch User Registration")
        .body(message)
        .unwrap();

    let credentials = Credentials::new(smtp_user, smtp_password);

    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(credentials)
        .build();

    match mailer.send(&email) {
        Ok(_) => debug!("Email sent successfully"),
        Err(e) => error!("Could not send email: {:?}", e),
    }
}

#[tokio::test]
async fn test_email_client() {
    let test_message = "test".to_string();
    send_registration_mail(test_message).await
}
