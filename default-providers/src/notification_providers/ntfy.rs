use state_management::{Notification, NotificationReason, StateHandle};

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Config {
    base: String,
    topic: String,
    title: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    priority: Option<u8>,
    click: Option<url::Url>,
    attach: Option<url::Url>,
    markdown: Option<bool>,
    icon: Option<url::Url>,
    filename: Option<String>,
    delay: Option<String>,
    email: Option<String>,
    call: Option<String>,
}
#[derive(serde::Serialize)]
struct NotificationBody {
    topic: String,
    message: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    priority: Option<u8>,
    #[serde(skip_serializing_if="Option::is_none")]
    click: Option<url::Url>,
    #[serde(skip_serializing_if="Option::is_none")]
    attach: Option<url::Url>,
    #[serde(skip_serializing_if="Option::is_none")]
    markdown: Option<bool>,
    #[serde(skip_serializing_if="Option::is_none")]
    icon: Option<url::Url>,
    #[serde(skip_serializing_if="Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    delay: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    call: Option<String>,
}
impl From<&Config> for NotificationBody {
    fn from(value: &Config) -> Self {
        Self {
            topic: value.topic.clone(),
            message: None,
            title: value.title.clone(),
            tags: value.tags.clone(),
            priority: value.priority.clone(),
            click: value.click.clone(),
            attach: value.attach.clone(),
            markdown: value.markdown.clone(),
            icon: value.icon.clone(),
            filename: value.filename.clone(),
            delay: value.delay.clone(),
            email: value.email.clone(),
            call: value.call.clone(),
        }
    }
}

pub struct NtfyNotificationProvider {
    config: Config,
}
impl state_management::NotificationProvider for NtfyNotificationProvider {
    const ID: &'static str = "ntfy";
    type Config = Config;

    fn new(_state: StateHandle, config: Self::Config) -> Self {
        log::info!("registering ntfy notification provider");
        Self { config, }
    }

    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }

    fn send(&self, _source_id: String, notification: Notification) {
        let message = format!("{} {}", notification.item_name, match &notification.reason {
            NotificationReason::WentOnline => "went online",
            NotificationReason::WentOffline => "went offline",
            NotificationReason::Seen => "was seen",
            NotificationReason::Other(msg) => &*msg,
        });
        let mut body = NotificationBody::from(&self.config);
        body.message = Some(message);
        log::info!("sending notification to {}", self.config.base);
        tokio::spawn(reqwest::Client::new().post(&self.config.base)
            .json(&body)
            .send());
    }
}