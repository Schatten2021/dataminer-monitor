mod status_provider_wrapper;
mod notification_provider_wrapper;

use std::collections::{HashMap, HashSet};
use status_provider_wrapper::StatusProvider;
use notification_provider_wrapper::NotificationProvider;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
struct Config {
    #[serde(default)]
    status: HashMap<String, toml::Value>,
    #[serde(default)]
    notifications: HashMap<String, toml::Value>,
    #[serde(default)]
    disabled: DisabledConfig,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
struct DisabledConfig {
    #[serde(default)]
    status: HashSet<String>,
    #[serde(default)]
    notifications: HashSet<String>,
}

pub(crate) struct State {
    config: Config,
    #[cfg(feature = "rocket-integration")]
    tokio_runtime_handle: rocket::tokio::runtime::Handle,
    status_providers: HashMap<String, Box<dyn StatusProvider>>,
    notification_providers: HashMap<String, std::sync::Arc<parking_lot::RwLock<dyn NotificationProvider>>>,
}
impl State {
    fn load_config(path: impl AsRef<std::path::Path>) -> Config {
        toml::from_str(&std::fs::read_to_string(path).unwrap_or_default())
            .unwrap_or_default()
    }
    pub(crate) fn create(config_path: impl AsRef<std::path::Path>) -> Self {
        #[cfg(feature = "rocket-integration")]
        let tokio_runtime_handle = rocket::tokio::runtime::Handle::current();
        Self {
            config: Self::load_config(config_path),
            #[cfg(feature = "rocket-integration")]
            tokio_runtime_handle,
            status_providers: HashMap::new(),
            notification_providers: HashMap::new(),
        }
    }
    pub(crate) fn register_status_provider<T: crate::StatusProvider>(&mut self, handle: crate::StateHandle) -> Result<(), ()> {
        if self.status_providers.contains_key(T::ID) { return Err(()) }
        if self.config.disabled.status.contains(T::ID) { return Ok(()) }
        let config = self.config.status.get(T::ID).cloned();
        let provider = <T as crate::StatusProvider>::new(handle, config.map(<T::Config as serde::Deserialize>::deserialize).map(Result::unwrap_or_default).unwrap_or_default());
        self.status_providers.insert(T::ID.to_string(), Box::new(provider));
        Ok(())
    }
    pub(crate) fn unregister_status_provider(&mut self, id: &str) {
        self.status_providers.remove(id);
    }

    pub(crate) fn register_notification_provider<T: crate::NotificationProvider>(&mut self, handle: crate::StateHandle) -> Result<(), ()> {
        if self.notification_providers.contains_key(T::ID) { return Err(()) }
        if self.config.disabled.notifications.contains(T::ID) { return Ok(()); }
        let config = self.config.notifications.get(T::ID).cloned();
        let provider = <T as crate::NotificationProvider>::new(handle, config.map(<T::Config as serde::Deserialize>::deserialize).map(Result::unwrap_or_default).unwrap_or_default());
        self.notification_providers.insert(T::ID.to_string(), std::sync::Arc::new(parking_lot::RwLock::new(provider)));
        Ok(())
    }
    pub(crate) fn unregister_notification_provider(&mut self, id: &str) {
        self.notification_providers.remove(id);
    }

    pub(crate) fn reload_config(&mut self, path: impl AsRef<std::path::Path>) {
        self.config = Self::load_config(path);
        for status_provider_id in &self.config.disabled.status {
            self.status_providers.remove(status_provider_id);
        }
        for notification_provider_id in &self.config.disabled.notifications {
            self.notification_providers.remove(notification_provider_id);
        }
        for (id, status_provider) in self.status_providers.iter_mut() {
            status_provider.reconfigure(self.config.status.get(id).cloned());
        }
        for (id, notification_provider) in self.notification_providers.iter_mut() {
            notification_provider.write().reconfigure(self.config.notifications.get(id).cloned());
        }
    }
    pub(crate) fn send_notification(&self, source_type_id: &str, message: crate::notifications::Notification) {
        let source_type_id = source_type_id.to_string();
        #[cfg(feature = "rocket-integration")]
        let _tokio_guard = self.tokio_runtime_handle.enter();
        self.notification_providers.values()
            .for_each(|p| {
                let p = p.clone();
                let source_type_id = source_type_id.clone();
                let message = message.clone();
                self.tokio_runtime_handle.spawn(async move {
                    p.read().send(source_type_id.clone(), message.clone())
                });
            });
    }
    pub(crate) fn get_all_stati(&self) -> HashMap<String, HashMap<String, crate::Status>> {
        self.status_providers.iter()
            .map(|(id, v)| (id.clone(), v.all_stati()))
            .collect()
    }
}
#[cfg(feature = "rocket-integration")]
#[rocket::async_trait]
impl rocket::route::Handler for crate::State {
    async fn handle<'r>(&self, request: &'r rocket::Request<'_>, data: rocket::Data<'r>) -> rocket::route::Outcome<'r> {
        let mut path = request.routed_segments(0..);
        if path.len() == 1 && path.get(0) == Some("reload_config") {
            self.0.write().reload_config(Self::DEFAULT_CONFIG_PATH);
            return rocket::route::Outcome::Success(rocket::Response::build()
                .status(rocket::http::Status::MovedPermanently)
                .raw_header("Location", "/")
                .finalize());
        }
        let original_path = path.clone();

        let mut data = match path.next() {
            None => data,
            Some(provider_id) => {
                let lock = self.0.read();
                match lock.status_providers.get(provider_id) {
                    None => data,
                    Some(provider) => match provider.handle_request(path.clone(), request, data) {
                        rocket::route::Outcome::Forward((data, _)) => data,
                        other => return other,
                    }
                }
            }
        };
        for provider in self.0.read().notification_providers.values() {
            match provider.read().handle_request(original_path.clone(), request, data) {
                rocket::route::Outcome::Forward((result_data, _status)) => {data = result_data},
                v => return v,
            }
        }
        rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}
#[cfg(feature = "rocket-integration")]
impl Into<Vec<rocket::Route>> for crate::State {
    fn into(self) -> Vec<rocket::Route> {
        use rocket::http::Method::*;
        [Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch]
            .map(|method| rocket::Route::new(method, "/<path..>", self.clone()))
            .to_vec()
    }
}