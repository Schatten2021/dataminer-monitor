fn default_static_dir() -> std::path::PathBuf { std::path::PathBuf::from("static/") }

#[cfg(not(any(feature = "website-notification-provider-hot-reload", feature = "website-notification-provider-include-default-website")))]
compile_error!("requires a source of content to serve: either the default website (`website-notification-provider-include-default-website`) or a hot-reload website (`website-notification-provider-hot-reload`)");

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_static_dir")]
    pub static_dir: std::path::PathBuf,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            static_dir: default_static_dir(),
        }
    }
}

pub struct WebsiteNotificationProvider {
    config: Config,
}
impl state_management::NotificationProvider for WebsiteNotificationProvider {
    const ID: &'static str = "website";
    type Config = Config;
    fn new(state_handle: state_management::StateHandle, config: Self::Config) -> Self {
        state_handle.add_dependency_notification_provider::<crate::notification_providers::ApiNotificationProvider>();
        Self {
            config,
        }
    }
    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }
    #[allow(unused_variables)]
    fn send(&self, source_id: String, notification: state_management::Notification) {}
    fn handle_rocket_http_request<'r, 'l>(&self, path: rocket::http::uri::Segments<rocket::http::uri::fmt::Path>, request: &'r rocket::Request<'l>, data: rocket::Data<'r>) -> rocket::route::Outcome<'r> {
        use rocket::response::Responder;
        macro_rules! respond_with {
            ($what:ident($val:expr)) => {
                {
                    match rocket::response::content::$what($val).respond_to(request) {
                        Ok(response) => rocket::route::Outcome::Success(response),
                        Err(e) => rocket::route::Outcome::Error(e),
                    }
                }
            };
        }
        macro_rules! include_static {
            (String $path:literal | $dynamic_path:literal) => {
                include_static!((include_str!($path).to_string()) | read_to_string($dynamic_path): String)
            };
            (($include_val:expr) | $dynamic_read_fn:ident($dynamic_url:literal): $dtype:ty) => {
                {
                    #[allow(unused_assignments)]
                    let mut result: $dtype = <$dtype>::default();
                    #[cfg(feature = "website-notification-provider-include-default-website")]
                    {
                        result = $include_val;
                    }
                    #[cfg(feature = "website-notification-provider-hot-reload")]
                    if let Ok(content) = std::fs::$dynamic_read_fn(self.config.static_dir.join($dynamic_url)) {
                        result = content;
                    }
                    result
                }
            };
        }
        let path = &*path.collect::<Vec<_>>().join("/");
        match path {
            "" | "index.html" | "static/index.html" => respond_with!(RawHtml(include_static!(String "../../../static/index.html" | "index.html"))),
            "static/style.css" => respond_with!(RawCss(include_static!(String "../../../static/style.css" | "style.css"))),
            "static/wasm/frontend.js" => respond_with!(RawJavaScript(include_static!(String "../../../static/wasm/frontend.js" | "wasm/frontend.js"))),
            "static/wasm/frontend_bg.wasm" => {
                #[derive(rocket::Responder)]
                #[response(status = 200, content_type = "application/wasm")]
                struct Responder(Vec<u8>);
                let val = include_static!((include_bytes!("../../../static/wasm/frontend_bg.wasm").to_vec()) | read("wasm/frontend_bg.wasm"): Vec<u8>);
                match Responder(val).respond_to(request) {
                    Ok(response) => rocket::route::Outcome::Success(response),
                    Err(e) => rocket::route::Outcome::Error(e),
                }
            }
            _ => {
                rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
            }
        }
    }
}