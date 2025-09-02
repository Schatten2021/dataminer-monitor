use std::collections::HashMap;
use rocket::{Data, Request};
use rocket::route::Outcome;

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Config {
    #[serde(default)]
    status: HashMap<String, toml::Value>,
    #[serde(default)]
    notifications: HashMap<String, toml::Value>,
}

pub(crate) struct State {
    config: Config,
    handle: rocket::tokio::runtime::Handle,
    status_providers: HashMap<String, Box<dyn StatusProvider>>,
    notification_providers: HashMap<String, Box<dyn NotificationProvider>>,
}
use dyn_types::*;
mod dyn_types {
    macro_rules! parse_toml {
        ($toml:ident => $target:ty) => {
            $toml.map(<$target as serde::Deserialize>::deserialize).map(Result::unwrap).unwrap_or_default()
        };
    }
    macro_rules! passthrough {
        ($original:ident::$func:ident(&self$(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
            fn $func(&self $(, $arg_name: $arg_ty)*) -> $ret_ty {
                crate::$original::$func(self $(, $arg_name)*)
            }
        };
    }
    pub(super) use parse_toml;
    use crate::Status;

    //noinspection DuplicatedCode
    pub(super) trait StatusProvider: Send + Sync {
        fn reconfigure(&mut self, config: Option<toml::Value>);
        fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r>;
        fn all_stati(&self) -> std::collections::HashMap<String, Status>;
    }
    impl<T: crate::StatusProvider> StatusProvider for T {
        fn reconfigure(&mut self, config: Option<toml::Value>) {
            crate::StatusProvider::reconfigure(self, parse_toml!(config => T::Config))
        }
        fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
            crate::StatusProvider::handle_request(self, path, request, data)
        }
        passthrough!(StatusProvider::all_stati(&self) -> std::collections::HashMap<String, crate::Status>);
    }
    //noinspection DuplicatedCode
    pub(super) trait NotificationProvider: Send + Sync {
        fn reconfigure(&mut self, config: Option<toml::Value>);
        fn send(&self, source_type_id: String, notification: crate::Notification);
        fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r>;
    }
    impl<T: crate::NotificationProvider> NotificationProvider for T {
        fn reconfigure(&mut self, config: Option<toml::Value>) {
            crate::NotificationProvider::reconfigure(self, parse_toml!(config => T::Config));
        }
        fn send(&self, source_type_id: String, notification: crate::Notification) {
            crate::NotificationProvider::send(self, source_type_id, notification);
        }
        fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
            crate::NotificationProvider::handle_request(self, path, request, data)
        }
    }
}

impl State {
    const CONFIG_PATH: &'static str = "config.toml";
    fn load_config() -> Config {
        let config_string = std::fs::read_to_string(Self::CONFIG_PATH).expect("failed to read config file");
        toml::from_str(&config_string).expect("failed to parse config file")
    }
    pub(crate) fn new() -> Self {
        Self {
            config: Self::load_config(),
            ..Default::default()
        }
    }
    pub(crate) fn register_status_provider<T: crate::StatusProvider>(&mut self, handle: crate::StateHandle) -> Result<(), ()> {
        let id = T::ID;
        if self.status_providers.contains_key(id) { return Err(()); }
        let config = self.config.status.get(T::ID).cloned();
        let provider = <T as crate::StatusProvider>::new(handle, parse_toml!(config => T::Config));
        self.status_providers.insert(id.to_string(), Box::new(provider));
        Ok(())
    }
    pub(crate) fn register_notification_provider<T: crate::NotificationProvider>(&mut self, handle: crate::StateHandle) -> Result<(), ()> {
        let id = T::ID;
        if self.notification_providers.contains_key(id) { return Err(())}
        let config = self.config.notifications.get(T::ID).cloned();
        let provider  = <T as crate::NotificationProvider>::new(handle, parse_toml!(config => T::Config));
        self.notification_providers.insert(id.to_string(), Box::new(provider));
        Ok(())
    }
    pub(crate) fn reload_config(&mut self) {
        self.config = Self::load_config();
        for (id, status_provider) in self.status_providers.iter_mut() {
            status_provider.reconfigure(self.config.status.get(id).cloned());
        }
        for (id, notification_provider) in self.notification_providers.iter_mut() {
            notification_provider.reconfigure(self.config.notifications.get(id).cloned());
        }
    }
    pub(crate) fn send_notification(&self, source_status_type_id: String, message: crate::Notification) {
        let guard = self.handle.enter();
        self.notification_providers.values()
            .for_each(|v| v.send(source_status_type_id.clone(), message.clone()));
        drop(guard);
    }
    pub(crate) fn get_all_stati(&self) -> HashMap<String, HashMap<String, crate::Status>> {
        self.status_providers.iter()
            .map(|(id, v)| (id.clone(), v.all_stati()))
            .collect()
    }
}
impl Default for State {
    fn default() -> Self {
        Self {
            config: Default::default(),
            status_providers: Default::default(),
            notification_providers: Default::default(),
            handle: rocket::tokio::runtime::Handle::current()
        }
    }
}
#[rocket::async_trait]
impl rocket::route::Handler for crate::State {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let mut path = request.routed_segments(0..);
        if path.len() == 1 && path.get(0).unwrap() == "reload_config" {
            self.0.write().reload_config();
            let response = rocket::response::Response::build()
                .status(rocket::http::Status::Ok)
                .header(rocket::http::Header::new("location", "/"))
                .finalize();
            return Outcome::Success(response)
        }
        let original_path = path.clone();
        let mut data = match path.next() {
            None => data,
            Some(provider) => {
                let lock = self.0.read();
                match lock.status_providers.get(provider) {
                    None => data,
                    Some(provider) => match provider.handle_request(path.clone(), request, data) {
                        Outcome::Forward((data, _status)) => data,
                        other => return other,
                    }
                }
            }
        };
        for provider in self.0.read().notification_providers.values() {
            match provider.handle_request(original_path.clone(), request, data) {
                Outcome::Forward((result_data, _status)) => {data = result_data},
                v => return v,
            }
        }
        Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}
impl Into<Vec<rocket::Route>> for crate::State {
    fn into(self) -> Vec<rocket::Route> {
        use rocket::http::Method::*;
        let mut routes = vec![];
        for method in [Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch] {
            routes.push(rocket::Route::new(method, "/<path..>", self.clone()))
        }
        routes
    }
}