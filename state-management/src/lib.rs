mod state;
pub mod standard_modules;


#[derive(Clone)]
pub struct State(std::sync::Arc<parking_lot::RwLock<state::State>>);
impl State {
    pub fn new() -> Self {
        Self(std::sync::Arc::new(parking_lot::RwLock::new(state::State::new())))
    }
    pub fn register_status_provider<T: StatusProvider>(&self) -> Result<Self, ()> {
        let handle = StateHandle(self.clone(), T::ID);
        self.0.write().register_status_provider::<T>(handle).map(|_| self.clone())
    }
    pub fn register_notification_provider<T: NotificationProvider>(&self) -> Result<Self, ()> {
        let handle = StateHandle(self.clone(), T::ID);
        self.0.write().register_notification_provider::<T>(handle).map(|_| self.clone())
    }
    pub fn send_notification(&self, source_status_type_id: String, message: Notification) {
        self.0.read().send_notification(source_status_type_id, message)
    }
    pub fn get_all_stati(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, Status>> {
        self.0.read().get_all_stati()
    }
}
#[derive(Clone)]
pub struct StateHandle(State, &'static str);
impl StateHandle {
    pub fn send(&self, message: Notification) {
        self.0.send_notification(self.1.to_string(), message)
    }
    pub fn get_all_stati(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, Status>> {
        self.0.get_all_stati()
    }
}
pub struct Status {
    pub name: String,
    pub is_online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Local>>
}

pub trait StatusProvider: 'static + Send + Sync {
    const ID: &'static str;
    const NAME: &'static str = Self::ID;
    type Config: serde::Serialize + for<'a> serde::Deserialize<'a> + Default;
    fn new(state: StateHandle, config: Self::Config) -> Self;
    fn reconfigure(&mut self, config: Self::Config);
    fn all_stati(&self) -> std::collections::HashMap<String, Status>;
    #[allow(unused_variables, unused_mut)]
    fn handle_request<'r, 'l>(&self, mut path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}
pub trait NotificationProvider: 'static + Send + Sync {
    const ID: &'static str;
    type Config: serde::Serialize + for<'a> serde::Deserialize<'a> + Default;
    fn new(state: StateHandle, config: Self::Config) -> Self;
    fn reconfigure(&mut self, config: Self::Config);
    fn send(&self, source_type: String, notification: Notification);
    #[allow(unused_variables, unused_mut)]
    fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}
#[derive(Clone, Debug)]
pub struct Notification {
    pub item_name: String,
    pub item_id: String,
    pub reason: NotificationReason,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotificationReason {
    WentOnline,
    WentOffline,
    Seen,
}
impl std::fmt::Display for NotificationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NotificationReason::WentOnline => "went online",
            NotificationReason::WentOffline => "went offline",
            NotificationReason::Seen => "was seen",
        }.fmt(f)
    }
}