pub trait StatusProvider: 'static + Send + Sync {
    const ID: &'static str;
    const NAME: &'static str;
    type Config: serde::Serialize + for<'de> serde::Deserialize<'de> + Default;
    fn new(state: crate::StateHandle, config: Self::Config) -> Self;
    fn update_config(&mut self, config: Self::Config);
    fn current_stati(&self) -> std::collections::HashMap<String, crate::Status>;
    #[cfg(feature = "rocket-integration")]
    #[allow(unused_variables)]
    fn handle_rocket_http_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}
pub trait NotificationProvider: 'static + Send + Sync {
    const ID: &'static str;
    type Config: serde::Serialize + for<'de> serde::Deserialize<'de> + Default;
    fn new(state: crate::StateHandle, config: Self::Config) -> Self;
    fn update_config(&mut self, config: Self::Config);
    fn send(&self, source_id: String, notification: crate::notifications::Notification);
    #[cfg(feature = "rocket-integration")]
    #[allow(unused_variables)]
    fn handle_rocket_http_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
    }
}