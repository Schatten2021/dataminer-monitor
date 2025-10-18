fn default_name() -> String { "No Reply".to_string() }

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Config {
    address: String,
    password: String,
    server: String,
    #[serde(default="default_name")]
    name: String,
    subscribers: Vec<String>,
}
impl Default for Config {
    fn default() -> Self {
        #[cfg(feature = "logging")]
        log::error!("No information provided for EmailNotificationProvider. Please add the required config to the config file.");
        Self {
            address: Default::default(),
            password: Default::default(),
            server: Default::default(),
            name: default_name(),
            subscribers: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct EmailNotificationProvider {
    config: Config,
    credentials: lettre::transport::smtp::authentication::Credentials,
}
impl state_management::NotificationProvider for EmailNotificationProvider {
    const ID: &'static str = "email";
    type Config = Config;

    fn new(_state: state_management::StateHandle, config: Self::Config) -> Self {
        Self {
            credentials: lettre::transport::smtp::authentication::Credentials::new(config.address.clone(), config.password.clone()),
            config,
        }
    }
    fn update_config(&mut self, config: Self::Config) {
        self.credentials = lettre::transport::smtp::authentication::Credentials::new(config.address.clone(), config.password.clone());
        self.config = config;
    }
    fn send(&self, source_id: String, notification: state_management::Notification) {
        if notification.reason == state_management::NotificationReason::Seen { return; }
        let cloned = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = cloned.send_message(
                format!("{} `{}` {}", source_id, notification.item_name, notification.reason),
                format!(r#"
                <h1> The {} <code>{}</code> just {}.</h1>
                <p>{}</p>
                "#, source_id, notification.item_name, notification.reason,
                match &notification.reason {
                    state_management::NotificationReason::WentOnline => "Everything is fine :)",
                    state_management::NotificationReason::WentOffline => "Might need to do something",
                    state_management::NotificationReason::Seen => "Probably Ok",
                    state_management::NotificationReason::Other(v) => v,
                })
            ) {
                #[cfg(feature = "logging")]
                log::error!("Failed to send notification for {} because: {:?}", notification.reason, e);
            }
        });
    }
}
impl EmailNotificationProvider {
    fn send_message(self, subject: String, body: impl lettre::message::IntoBody + Clone) -> Result<(), Box<dyn std::error::Error>> {
        use lettre::Transport;
        let mailer = lettre::transport::smtp::SmtpTransport::relay(&self.config.server)?
            .credentials(self.credentials)
            .build();
        let builder_preset = lettre::Message::builder()
            .from(format!("{} <{}>", self.config.name, self.config.address).parse()?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML);
        for target in self.config.subscribers {
            mailer.send(&builder_preset.clone()
                .to(target.parse()?)
                .body(body.clone())?)?;
        }
        Ok(())
    }
}