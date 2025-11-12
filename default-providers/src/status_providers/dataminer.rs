use parking_lot::RwLock;
use state_management::{Notification, StateHandle, Status};
use std::collections::HashMap;
use std::sync::Arc;
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::{Data, Request};
use rocket::route::Outcome;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(skip_serializing_if="Option::is_none")]
    name: Option<String>,
    timeout: chrono::Duration,
}
#[derive(Clone)]
struct MinerStatus {
    last_seen: chrono::DateTime<chrono::Local>,
    marked_offline: bool,
}

pub struct DataminerStatusProvider {
    config: HashMap<String, Config>,
    state_handle: StateHandle,
    current_stati: RwLock<HashMap<String, Arc<RwLock<MinerStatus>>>>,
}
impl state_management::StatusProvider for DataminerStatusProvider {
    const ID: &'static str = "miner";
    const NAME: &'static str = "Dataminer";
    type Config = HashMap<String, Config>;
    fn new(state: StateHandle, config: Self::Config) -> Self {
        Self {
            config,
            state_handle: state, current_stati: RwLock::new(HashMap::new())
        }
    }
    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }
    fn current_stati(&self) -> HashMap<String, Status> {
        #[cfg(feature = "logging")]
        log::debug!("gathering current stati from DataminerStatusProvider");
        // Note: the stati have to be combined from 2 different sources:
        // - all who have pinged already (from self.current_stati)
        // - all who have been manually configured (from self.config)
        let mut result = HashMap::new();
        // insert all miners whose status is found from the current_stati variable.
        for (id, status) in self.current_stati.read().iter() {
            let status = status.read().clone();
            let name = match self.config.get(id) {
                Some(v) => {
                    v.name.as_ref()
                        .unwrap_or(id)
                        .clone()
                }
                None => id.to_string()
            };
            result.insert(id.clone(), Status {
                name,
                is_online: !status.marked_offline,
                last_seen: Some(status.last_seen.to_utc()),
            });
        }
        #[cfg(feature = "logging")]
        log::trace!("found {} dataminers that have pinged already", result.len());
        for (id, status) in self.config.iter() {
            // deduplicate entries. Some may already have been in `self.current_stati`
            if result.contains_key(id) { continue }
            result.insert(id.clone(), Status {
                name: status.name.as_ref().unwrap_or_else(|| id).clone(),
                is_online: false,
                last_seen: None,
            });
        }
        #[cfg(feature = "logging")]
        log::trace!("found {} dataminers in total", result.len());
        result
    }
    fn handle_rocket_http_request<'r, 'l>(&self, mut path: Segments<Path>, request: &'r Request<'l>, data: Data<'r>) -> Outcome<'r> {
        if path.next() != Some("ping") || request.method() != rocket::http::Method::Post {
            return Outcome::Forward((data, rocket::http::Status::NotFound))
        }
        let id = match request.query_value::<String>("id") {
            Some(Ok(id)) => id,
            _ => return Outcome::Error(rocket::http::Status::BadRequest),
        };
        let config = self.config.get(&id).cloned();
        let name = match config {
            Some(config) => config.name.as_ref().unwrap_or(&id).clone(),
            None => id.clone()
        };
        #[cfg(feature = "logging")]
        log::trace!("received ping from dataminer {name}");
        macro_rules! send_notification {
            ($reason:ident) => {
                self.state_handle.send_notification(::state_management::Notification {
                    item_name: name.clone(),
                    item_id: id.clone(),
                    reason: ::state_management::NotificationReason::$reason,
                })
            };
        }
        send_notification!(Seen);
        if !self.current_stati.read().contains_key(&id) {
            #[cfg(feature = "logging")]
            log::debug!("dataminer {name} pinged for the first time");
            self.current_stati.write().insert(id.clone(), Arc::new(RwLock::new(MinerStatus {
                last_seen: chrono::Local::now(),
                marked_offline: false,
            })));
            send_notification!(WentOnline);
        }
        let status = self.current_stati.read().get(&id).cloned().unwrap();
        let mut lock = status.write();
        let went_online = lock.marked_offline;
        let timestamp = chrono::Local::now();
        lock.marked_offline = false;
        lock.last_seen = timestamp;
        drop(lock);
        // Note that because new miners are inserted with `marked_offline: false`, this does not run the risk of sending the WentOnline message twice.
        if went_online {
            #[cfg(feature = "logging")]
            log::info!("dataminer {name} went back online");
            send_notification!(WentOnline);
        }
        if let Some(config) = self.config.get(&id) {
            (|| {
                let Ok(timeout) = config.timeout.to_std() else { return };
                let handle = self.state_handle.clone();
                let status = status.clone();
                rocket::tokio::spawn(async move {
                    rocket::tokio::time::sleep(timeout).await;
                    if status.read().last_seen == timestamp {
                        status.write().marked_offline = true;
                        #[cfg(feature = "logging")]
                        log::info!("dataminer {name} went offline");
                        handle.send_notification(Notification {
                            item_name: name,
                            item_id: id,
                            reason: state_management::NotificationReason::WentOffline,
                        })
                    }
                });
            })()
        }
        Outcome::Success(rocket::Response::new())
    }
}