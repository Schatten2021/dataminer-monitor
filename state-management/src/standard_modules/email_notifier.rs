
use crate::{Notification, NotificationReason, StateHandle};
fn default_name() -> String { "No Reply".to_string() }
#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct Config {
    address: String,
    password: String,
    server: String,
    #[serde(default="default_name")]
    name: String,
    subscribers: Vec<String>,
}

#[derive(Clone)]
pub struct EmailNotifier {
    config: Config,
    credentials: lettre::transport::smtp::authentication::Credentials,
}
impl crate::NotificationProvider for EmailNotifier {
    const ID: &'static str = "email";
    type Config = Config;

    fn new(_state: StateHandle, config: Self::Config) -> Self {
        Self {
            credentials: lettre::transport::smtp::authentication::Credentials::new(config.address.clone(), config.password.clone()),
            config,
        }
    }

    fn reconfigure(&mut self, config: Self::Config) {
        self.credentials = lettre::transport::smtp::authentication::Credentials::new(config.address.clone(), config.password.clone());
        self.config = config;
    }

    fn send(&self, source_type: String, notification: Notification) {
        if notification.reason == NotificationReason::Seen { return; }
        let cloned = self.clone();
        std::thread::spawn(move || {
            cloned.send(
                format!("{source_type} `{}` {}", notification.item_name, notification.reason),
                format!(r#"
    <h1> The {source_type} <code>{}</code> just {}.</h1>
    <p>{}</p>
    "#, notification.item_name, notification.reason, (notification.reason == NotificationReason::WentOnline).then_some("Everything's OK :)").unwrap_or("They might need help"))
            ).expect("unable to send E-Mail notification");
        });
    }
}
impl EmailNotifier {
    fn send(&self, subject: String, body: impl lettre::message::IntoBody + Clone) -> Result<(), Box<dyn std::error::Error>> {
        use lettre::Transport;
        let mailer = lettre::transport::smtp::SmtpTransport::starttls_relay(&self.config.server).expect("unable to send E-Mail")
            .credentials(self.credentials.clone())
            .build();
        let builder_preset = lettre::Message::builder()
            .from(format!("{} <{}>", self.config.name, self.config.address).parse()?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML);
        for target in self.config.subscribers.iter() {
            mailer.send(&builder_preset.clone()
                .to(target.parse().expect("failed to parse target E-Mail address"))
                .body(body.clone())?)?;
        }
        Ok(())
    }
}