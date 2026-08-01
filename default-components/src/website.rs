use std::collections::{HashMap};
use tokio::time::MissedTickBehavior;
use server::{AttributeValue, ComponentHandle};
use utils::Never;
use crate::filters::SingleFilter;

const fn hourly() -> chrono::Duration { chrono::Duration::hours(1) }

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    url: String,
    #[serde(default="hourly")]
    interval: chrono::Duration,
    #[serde(default)]
    status: SingleFilter<u16>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            url: "https://example.com".to_string(),
            interval: hourly(),
            status: SingleFilter::default()
        }
    }
}
pub struct WebsiteStatuse {
    config: HashMap<String, Config>,
    task_handles: HashMap<String, tokio::task::JoinHandle<()>>,
    state: ComponentHandle,
}
impl server::Component for WebsiteStatuse {
    const ID: &'static str = "website";
    type Config = HashMap<String, Config>;
    type ConfigError = Never;

    fn init(server: ComponentHandle, config: Self::Config) -> Result<Self, Self::ConfigError> {
        Ok(Self {
            task_handles: config.iter()
                .map(|(a, b)| (a.clone(), b.clone()))
                .map(|(id, config)| {
                    let handle = server.clone();
                    (id.clone(), spawn_listen_task(id, config, handle))
                }).collect(),
            config,
            state: server,
        })
    }

    fn reconfigure(&mut self, config: Self::Config) -> Result<(), Self::ConfigError> {
        for id in self.config.keys()
            .filter(|k| !config.contains_key(*k))
            .cloned()
            .collect::<Vec<_>>() {
            self.task_handles.remove(&id)
                .map(|h| h.abort());
            self.config.remove(&id);
        }
        for (id, new_config) in config.into_iter()
            .filter(|(id, new_conf)| self.config.get(id)
                .map(|old_conf| old_conf != new_conf)
                .unwrap_or(true))
            .collect::<Vec<_>>()
        {
            let handle = self.state.clone();
            self.config.insert(id.clone(), new_config.clone());
            self.task_handles.insert(id.clone(), spawn_listen_task(id, new_config, handle))
                .map(|old| old.abort());
        }
        Ok(())
    }
}
fn spawn_listen_task(id: String, config: Config, state: ComponentHandle) -> tokio::task::JoinHandle<()> {
    let mut ticker = tokio::time::interval(config.interval.to_std().expect("couldn't convert interval to std interval"));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    tokio::task::spawn(async move {
        let client = reqwest::Client::new();
        loop {
            ticker.tick().await;
            let new_status = matches!(request_website(&client, &config).await, Ok(true));
            if Some(new_status) != state.get_online_state(&id) {
                state.change_online_state(&id, new_status);
            }
            if new_status {
                state.change_attribute(&id, "last_seen", AttributeValue::Date(chrono::Utc::now()))
            }
        }
    })
}
async fn request_website(client: &reqwest::Client, config: &Config) -> Result<bool, ()> {
    Ok(config.status.allows(&client.get(&config.url)
        .send().await.map_err(|e| {
        error!("couldn't request {:?}: {e}", config.url);
    })?
        .status().as_u16()))
}