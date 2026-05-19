use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use thiserror::Error;
use zeroclaw_core::SmtpConfig;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("email address error: {0}")]
    Address(#[from] lettre::address::AddressError),

    #[error("email message build error: {0}")]
    Message(#[from] lettre::error::Error),

    #[error("SMTP transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
}

pub async fn send_email(
    smtp: &SmtpConfig,
    recipient: &str,
    subject: &str,
    body: String,
) -> Result<(), EmailError> {
    let from: Mailbox = smtp.from().parse()?;
    let to: Mailbox = recipient.parse()?;
    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body)?;

    let credentials = Credentials::new(smtp.username().to_owned(), smtp.password().to_owned());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp.host())?
        .port(smtp.port())
        .credentials(credentials)
        .build();

    mailer.send(message).await?;

    Ok(())
}
