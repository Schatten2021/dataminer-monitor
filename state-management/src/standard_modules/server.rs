use std::collections::HashMap;
use std::sync::Arc;
use rocket::tokio;
use crate::{StateHandle, Status};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    url: String,
    interval: chrono::Duration,
}
struct ServerState {
    pub last_seen: Option<chrono::DateTime<chrono::Local>>,
    pub is_online: bool,
}

pub struct WebServerInfoProvider {
    config: HashMap<String, ServerConfig>,
    states: HashMap<String, Arc<parking_lot::RwLock<ServerState>>>,
    task_handles: HashMap<String, rocket::tokio::task::JoinHandle<()>>,
    handle: StateHandle,
}
impl crate::StatusProvider for WebServerInfoProvider {
    const ID: &'static str = "webserver";
    type Config = HashMap<String, ServerConfig>;

    fn new(state: StateHandle, config: Self::Config) -> Self {
        let (task_handles, states) = config.iter()
            .map(|(id, config)| {
                let status = Arc::new(parking_lot::RwLock::new(ServerState {
                    last_seen: None, is_online: false
                }));
                let ticker = rocket::tokio::time::interval(config.interval.to_std().unwrap());
               ((id.clone(), rocket::tokio::task::spawn(start_listening(ticker, id.clone(), config.clone(), status.clone(), state.clone()))), (id.clone(), status))
            })
            .collect();
        Self {
            config,
            states,
            task_handles,
            handle: state
        }
    }

    fn reconfigure(&mut self, config: Self::Config) {
        for id in self.config.keys().cloned().collect::<Vec<_>>().into_iter() {
            if !config.contains_key(&id) {
                self.config.remove(&id);
                self.states.remove(&id);
            }

        }
        for (id, new_config) in config.iter() {
            match self.config.get_mut(id) {
                Some(old_config) => {
                    if old_config == new_config { continue; }
                    self.config.insert(id.clone(), new_config.clone());
                    let ticker = rocket::tokio::time::interval(new_config.interval.to_std().unwrap());
                    let status = self.states[id].clone();
                    // start a new task and abort the old one. No need to change the state (in fact changing it might have undesired effects).
                    self.task_handles.insert(id.clone(), tokio::spawn(start_listening(ticker, id.clone(), new_config.clone(), status, self.handle.clone())))
                        .unwrap().abort();
                },
                None => {
                    let status = Arc::new(parking_lot::RwLock::new(ServerState {
                        last_seen: None, is_online: false
                    }));
                    let ticker = rocket::tokio::time::interval(new_config.interval.to_std().unwrap());
                    self.config.insert(id.clone(), new_config.clone());
                    self.states.insert(id.clone(), status.clone());
                    self.task_handles.insert(id.clone(), tokio::spawn(start_listening(ticker, id.clone(), new_config.clone(), status, self.handle.clone())));
                }
            }
        }
        self.config = config;
    }

    fn all_stati(&self) -> HashMap<String, Status> {
        self.states.iter()
            .map(|(id, state)| (id.clone(), Status {
                name: self.config[id].name.clone().unwrap_or_else(|| id.clone()),
                is_online: state.read().is_online,
                last_seen: state.read().last_seen,
            }))
            .collect()
    }
}
async fn start_listening(mut ticker: rocket::tokio::time::Interval, id: String, config: ServerConfig, status: Arc<parking_lot::RwLock<ServerState>>, state: StateHandle) {
    loop {
        ticker.tick().await;
        match (async || {
            let Ok(response) = reqwest::get(&config.url).await else {return Err(())};
            response.error_for_status().map_err(drop).map(drop)
        })().await {
            Ok(()) => {
                let mut lock = status.write();
                lock.last_seen = Some(chrono::Local::now());
                if !lock.is_online {
                    lock.is_online = true;
                    drop(lock);
                    state.send(crate::Notification {
                        item_name: config.name.clone().unwrap_or_else(|| id.clone()),
                        item_id: id.clone(),
                        reason: crate::NotificationReason::WentOnline,
                    })
                }
                state.send(crate::Notification {
                    item_name: config.name.clone().unwrap_or_else(|| id.clone()),
                    item_id: id.clone(),
                    reason: crate::NotificationReason::Seen,
                })
            }
            Err(()) => {
                if status.read().is_online {
                    status.write().is_online = false;
                    state.send(crate::Notification {
                        item_name: config.name.clone().unwrap_or_else(|| id.clone()),
                        item_id: id.clone(),
                        reason: crate::NotificationReason::WentOffline,
                    })
                }
            }
        }
    }
}