pub(super) trait NotificationProvider: Send + Sync {
    fn reconfigure(&mut self, config: Option<toml::Value>);
    fn send(&self, source_type_id: String, notification: crate::notifications::Notification);
    #[cfg(feature = "rocket-integration")]
    fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r>;
}
impl<T: crate::NotificationProvider> NotificationProvider for T {
    fn reconfigure(&mut self, config: Option<toml::Value>) {
        let config: <T as crate::NotificationProvider>::Config = config.map(<T::Config as serde::Deserialize>::deserialize)
            .map(Result::unwrap_or_default)
            .unwrap_or_default();
        <T as crate::NotificationProvider>::update_config(self, config);
    }
    fn send(&self, source_type_id: String, notification: crate::notifications::Notification) {
        <T as crate::NotificationProvider>::send(self, source_type_id, notification)
    }
    #[cfg(feature = "rocket-integration")]
    fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        <T as crate::NotificationProvider>::handle_rocket_http_request(self, path, request, data)
    }
}