use std::collections::HashMap;
use std::sync::Arc;
use log::info;
use parking_lot::RwLock;
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::{Data, Request};
use rocket::http::Status;
use rocket::route::Outcome;
use crate::{Notification, NotificationReason, StateHandle};


pub struct DataMinerInfoSource {
    config: HashMap<String, MinerConfig>,
    stati: RwLock<HashMap<String, Arc<RwLock<MinerStatus>>>>,
    state_handle: StateHandle,
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MinerConfig {
    pub name: Option<String>,
    pub timeout: chrono::Duration,
}
struct MinerStatus {
    last_seen: chrono::DateTime<chrono::Local>,
    marked_offline: bool,
}

impl crate::StatusProvider for DataMinerInfoSource {
    const ID: &'static str = "miner";
    type Config = HashMap<String, MinerConfig>;

    fn new(state: StateHandle, config: Self::Config) -> Self {
        Self {
            config,
            stati: Default::default(),
            state_handle: state,
        }
    }
    fn reconfigure(&mut self, config: Self::Config) {
        self.config = config;
    }
    fn handle_request<'r, 'l>(&self, mut path: Segments<Path>, request: &'r Request<'l>, data: Data<'r>) -> Outcome<'r> {
        if path.next() != Some("ping") || request.method() != rocket::http::Method::Post {
            return Outcome::Forward((data, Status::NotFound))
        }
        let id: String = match request.query_value("id") {
            Some(Ok(id)) => id,
            Some(Err(_e)) => return Outcome::Error(Status::BadRequest),
            None => return Outcome::Error(Status::BadRequest),
        };
        let name = self.config.get(&id).map(|v| v.name.clone().unwrap_or(id.clone())).unwrap_or(id.clone());
        self.state_handle.send(Notification {
            item_name: name.clone(),
            item_id: id.clone(),
            reason: NotificationReason::Seen,
        });
        let is_registered = self.stati.read().contains_key(&id);
        if !is_registered {
            self.stati.write().insert(id.clone(), Arc::new(RwLock::new(MinerStatus {
                last_seen: chrono::Local::now(),
                marked_offline: false,
            })));
            self.state_handle.send(Notification {
                item_name: self.config.get(&id).map(|v| v.name.clone().unwrap_or(id.clone())).unwrap_or(id.clone()).clone(),
                item_id: id.clone(),
                reason: NotificationReason::WentOnline,
            });
            return Outcome::Success(rocket::Response::new());
        }
        let lock = self.stati.read();
        let Some(status) = lock.get(&id) else {unreachable!("checked that the status exists above")};
        let mut lock = status.write();
        let went_online = lock.marked_offline;
        let timestamp = chrono::Local::now();
        lock.marked_offline = false;
        lock.last_seen = timestamp;
        drop(lock);
        if went_online {
            self.state_handle.send(Notification {
                item_name: name.clone(),
                item_id: id.clone(),
                reason: NotificationReason::WentOnline,
            })
        }
        if let Some(config) = self.config.get(&id) {
            let handle = self.state_handle.clone();
            let status = status.clone();
            let config = config.clone();
            std::thread::spawn(move || {
                std::thread::sleep(config.timeout.to_std().unwrap());
                if status.read().last_seen == timestamp { // The miner has not yet pinged back
                    status.write().marked_offline = true;
                    handle.send(Notification {
                        item_name: name.clone(),
                        item_id: id.clone(),
                        reason: NotificationReason::WentOffline,
                    })
                }
            });
        }
        Outcome::Success(rocket::Response::new())
    }
    fn all_stati(&self) -> HashMap<String, crate::Status> {
        let mut result = HashMap::new();
        for (id, status) in self.stati.read().iter() {
            let status = status.read();
            result.insert(id.clone(), crate::Status {
                name: id.clone(),
                is_online: !status.marked_offline,
                last_seen: Some(status.last_seen),
            });
        }
        for (id, config) in self.config.iter() {
            match result.get_mut(id) {
                None => {
                    result.insert(id.clone(), crate::Status {
                        name: config.name.clone().unwrap_or_else(|| id.clone()),
                        is_online: false,
                        last_seen: None,
                    });
                }
                Some(v) => {
                    if let Some(name) = &config.name {
                        v.name = name.clone();
                    }
                }
            }
        }
        result
    }
}