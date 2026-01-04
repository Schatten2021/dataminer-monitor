use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use state_management::Status;

macro_rules! debug {
    ($msg:literal $(,$args:expr)*) => {crate::debug!(status "website": $msg $(,$args)*)};
}
macro_rules! trace {
    ($msg:literal $(,$args:expr)*) => {crate::trace!(status "website": $msg $(,$args)*)};
}
macro_rules! info {
    ($msg:literal $(,$args:expr)*) => {crate::info!(status "website": $msg $(,$args)*)};
}

const fn hourly() -> chrono::Duration { chrono::Duration::hours(1) }

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    url: String,
    #[serde(default="hourly")]
    interval: chrono::Duration,
    #[serde(default, skip_serializing_if = "HashSet::is_empty", alias="accept", alias="accepted")]
    #[serde(alias="accepted-stati", alias="accept_stati", alias="accept-stati")]
    #[serde(alias="accepted-statuses", alias="accept-statuses", alias="accepted_statuses", alias="accept_statuses", alias="accept_statuses")]
    accepted_stati: HashSet<u16>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty", alias="reject", alias = "rejected")]
    #[serde(alias="rejected-stati", alias="reject_stati", alias="reject-stati")]
    #[serde(alias="rejected-statuses", alias="reject-statuses", alias="rejected_statuses", alias="reject_statuses", alias="reject_statuses")]
    rejected_stati: HashSet<u16>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            name: None,
            url: "https://example.com".to_string(),
            interval: hourly(),
            accepted_stati: Default::default(),
            rejected_stati: Default::default(),
        }
    }
}
struct ServerState {
    last_seen: Option<chrono::DateTime<chrono::Local>>,
    is_online: bool,
}
pub struct ServerStatusProvider {
    config: HashMap<String, Config>,
    states: HashMap<String, std::sync::Arc<parking_lot::RwLock<ServerState>>>,
    task_handles: HashMap<String, tokio::task::JoinHandle<()>>,
    state_handle: state_management::StateHandle,
}
impl state_management::StatusProvider for ServerStatusProvider {
    const ID: &'static str = "webserver";
    const NAME: &'static str = "Websites";
    type Config = HashMap<String, Config>;
    fn new(state_handle: state_management::StateHandle, config: Self::Config) -> Self {
        let (task_handles, states) = config.iter()
            .map(|(id, config)| {
                let status = std::sync::Arc::new(parking_lot::RwLock::new(ServerState {
                    last_seen: None,
                    is_online: false,
                }));
                let handle_data = (id.clone(), spawn_listen_task(id.clone(), config.clone(), status.clone(), state_handle.clone()));
                let state_data = (id.clone(), status);
                (handle_data, state_data)
            })
            .collect();
        Self {
            config,
            states,
            task_handles,
            state_handle,
        }
    }
    fn update_config(&mut self, config: Self::Config) {
        for id in self.config.keys()
            .filter(|k| !self.config.contains_key(*k))
            .cloned()
            .collect::<Vec<_>>() {
            let id = &id;
            info!("removing ping task for {}", id);
            self.config.remove(id);
            self.states.remove(id);
            self.task_handles.remove(id).map(|h| h.abort());
        }
        for (id, new_config) in config.into_iter()
            .filter(|(id, new_conf)| self.config.get(id).map(|old_conf| old_conf != new_conf).unwrap_or(true))
            .collect::<Vec<_>>() {
            self.config.insert(id.clone(), new_config.clone());
            match (self.task_handles.get_mut(&id), self.states.get(&id).cloned()) {
                (Some(handle), Some(state)) => {
                    info!("restarting ping task for {}", id);
                    handle.abort();
                    *handle = spawn_listen_task(id, new_config, state, self.state_handle.clone());
                }
                (None, None) => {
                    info!("starting ping task for {}", id);
                    let status = Arc::new(RwLock::new(ServerState { last_seen: None, is_online: false }));
                    self.states.insert(id.clone(), status.clone());
                    self.task_handles.insert(id.clone(), spawn_listen_task(id, new_config, status, self.state_handle.clone()));
                }
                (Some(_), None) | (None, Some(_)) => unreachable!("There should not be a task handle with no state and no state without a task handle!"),
            }
        }
    }
    fn current_stati(&self) -> HashMap<String, Status> {
        self.states.iter()
            .map(|(id, state)| {(id.clone(), Status {
                name: self.config[id].name.clone().unwrap_or_else(|| id.clone()),
                is_online: state.read().is_online,
                last_seen: state.read().last_seen.as_ref().map(chrono::DateTime::to_utc),
            })})
            .collect()
    }
}
fn spawn_listen_task(id: String, config: Config, status: std::sync::Arc<parking_lot::RwLock<ServerState>>, state_handle: state_management::StateHandle) -> tokio::task::JoinHandle<()> {
    let mut ticker = tokio::time::interval(config.interval.to_std().unwrap_or_else(|_| {core::time::Duration::new(3600,0)}));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let name = config.name.clone().unwrap_or_else(|| id.clone());
    debug!("spawning ping task for {}", name);
    macro_rules! send_notification {
        ($reason:ident($id:ident)) => {
            state_handle.send_notification(::state_management::Notification {
                item_name: name.clone(),
                item_id: $id.clone(),
                reason: ::state_management::NotificationReason::$reason,
            })
        };
    }
    tokio::task::spawn(async move {
        loop {
            ticker.tick().await;
            match (async || {
                trace!("testing status of webserver {} by pinging {}", name, config.url);
                let response = reqwest::get(&config.url).await.map_err(drop)?;
                let status = response.status().as_u16();
                if config.accepted_stati.contains(&status) {
                    return Ok(());
                }
                if config.rejected_stati.contains(&status) {
                    return Err(());
                }
                response.error_for_status().map_err(drop).map(drop)
            })().await {
                Ok(()) => {
                    let mut lock = status.write();
                    lock.last_seen = Some(chrono::Local::now());
                    if !lock.is_online {
                        lock.is_online = true;
                        drop(lock);
                        send_notification!(WentOnline(id));
                    }
                    send_notification!(Seen(id));
                }
                Err(()) => {
                    if status.read().is_online {
                        status.write().is_online = false;
                        send_notification!(WentOffline(id));
                    }
                }
            }
        }
    })
}
impl Drop for ServerStatusProvider {
    fn drop(&mut self) {
        for handle in self.task_handles.values() {
            handle.abort();
        }
    }
}