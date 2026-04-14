use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

#[derive(Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_display: String,
}

impl EmailConfig {
    pub fn from_env() -> Option<Self> {
        let host = std::env::var("SMTP_HOST").ok()?;
        let port: u16 = std::env::var("SMTP_PORT").ok()?.parse().ok()?;
        let user = std::env::var("SMTP_USER").ok()?;
        let pass = std::env::var("SMTP_PASS").ok()?;
        Some(Self {
            smtp_host: host,
            smtp_port: port,
            smtp_user: user.clone(),
            smtp_pass: pass,
            from_display: format!("QuestHub <{user}>"),
        })
    }
}

pub fn send_email(config: &EmailConfig, to: &str, subject: &str, body: &str) -> Result<(), String> {
    let email = Message::builder()
        .from(config.from_display.parse().map_err(|e| format!("{e}"))?)
        .to(to.parse().map_err(|e| format!("{e}"))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("{e}"))?;

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
    let mailer = SmtpTransport::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("{e}"))?
        .credentials(creds)
        .port(config.smtp_port)
        .build();

    mailer.send(&email).map_err(|e| format!("{e}"))?;
    Ok(())
}
