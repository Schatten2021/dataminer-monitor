pub(super) trait StatusProvider: Send + Sync {
    fn reconfigure(&mut self, config: Option<toml::Value>);
    fn all_stati(&self) -> std::collections::HashMap<String, crate::Status>;
    #[cfg(feature = "rocket-integration")]
    fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r>;
}
impl<T: crate::StatusProvider> StatusProvider for T {
    fn reconfigure(&mut self, config: Option<toml::Value>) {
        let config: <T as crate::StatusProvider>::Config = config.map(<T::Config as serde::Deserialize>::deserialize)
            .map(Result::unwrap_or_default)
            .unwrap_or_default();
        <T as crate::StatusProvider>::update_config(self, config);
    }
    fn all_stati(&self) -> std::collections::HashMap<String, crate::Status> {
        <T as crate::StatusProvider>::current_stati(self)
    }
    #[cfg(feature = "rocket-integration")]
    fn handle_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::request::Request<'l>, data: rocket::data::Data<'r>) -> rocket::route::Outcome<'r> {
        <T as crate::StatusProvider>::handle_rocket_http_request(self, path, request, data)
    }
}