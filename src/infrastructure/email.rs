use lettre::message::{Mailbox, SinglePart};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::rpc::json::ConnectError;
use axum::Json;
use axum::http::StatusCode;

#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from: String,
    pub base_url: String,
}

impl EmailConfig {
    pub fn from_env() -> Self {
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1025);
        let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "no-reply@local.test".to_string());
        let base_url =
            std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        Self {
            smtp_host,
            smtp_port,
            from,
            base_url,
        }
    }

    pub fn invite_url(&self, token: &str) -> String {
        format!(
            "{}/identity/accept-invite?token={}",
            self.base_url.trim_end_matches('/'),
            token
        )
    }
}

pub async fn send_invite_email(
    config: &EmailConfig,
    to_email: &str,
    store_name: &str,
    display_name: Option<&str>,
    role_name: Option<&str>,
    token: &str,
) -> Result<(), (StatusCode, Json<ConnectError>)> {
    let to = to_email.parse::<Mailbox>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::InvalidArgument,
                message: "invalid email address".to_string(),
            }),
        )
    })?;
    let from = config.from.parse::<Mailbox>().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
                message: "invalid SMTP_FROM".to_string(),
            }),
        )
    })?;

    let invite_url = config.invite_url(token);
    let name_line = display_name.filter(|v| !v.is_empty()).unwrap_or("there");
    let role_line = role_name.filter(|v| !v.is_empty()).unwrap_or("staff");

    let subject = format!("{}: staff invitation", store_name);
    let body = format!(
        "Hello {name},\n\nYou have been invited to {store} as {role}.\n\nAccept invite: {url}\n\nIf you did not expect this, you can ignore this email.\n",
        name = name_line,
        store = store_name,
        role = role_line,
        url = invite_url
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .singlepart(SinglePart::plain(body))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ConnectError {
                    code: crate::rpc::json::ErrorCode::Internal,
                    message: "failed to build email".to_string(),
                }),
            )
        })?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        .port(config.smtp_port)
        .build();

    mailer.send(email).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ConnectError {
                code: crate::rpc::json::ErrorCode::Internal,
                message: format!("failed to send email: {err}"),
            }),
        )
    })?;

    Ok(())
}
