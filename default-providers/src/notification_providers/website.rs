#[cfg(feature = "website-notification-provider-websockets")]
mod websockets;

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
    state_handle: state_management::StateHandle,
    #[cfg(feature = "website-notification-provider-websockets")]
    websockets: std::sync::Arc<parking_lot::RwLock<std::collections::HashSet<websockets::WebSocketRef>>>,
}
impl state_management::NotificationProvider for WebsiteNotificationProvider {
    const ID: &'static str = "website";
    type Config = Config;
    fn new(state_handle: state_management::StateHandle, config: Self::Config) -> Self {
        Self {
            config,
            state_handle,
            #[cfg(feature = "website-notification-provider-websockets")]
            websockets: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashSet::new())),
        }
    }
    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }
    #[allow(unused_variables)]
    fn send(&self, source_id: String, notification: state_management::Notification) {
        #[cfg(feature = "website-notification-provider-websockets")]
        {
            let message = match notification.reason {
                ::state_management::NotificationReason::Seen => ::api_types::WebSocketMessage::MinerPing { type_id: source_id, id: notification.item_id },
                ::state_management::NotificationReason::WentOnline | ::state_management::NotificationReason::WentOffline => ::api_types::WebSocketMessage::MinerStatusChange(::api_types::StatusUpdate {
                    type_id: source_id,
                    id: notification.item_id,
                    new_status: notification.reason == ::state_management::NotificationReason::WentOnline,
                }),
                ::state_management::NotificationReason::Other(msg) => {
                    #[cfg(feature = "logging")]
                    log::debug!("{msg} is not implemented");
                    return;
                },
            };
            let message = match rocket::serde::json::to_string(&message) {
                Ok(text) => text,
                Err(e) => {
                    #[cfg(feature = "logging")]
                    log::error!("unable to serialize message for sending: {e}");
                    return;
                }
            };
            self.websockets.read().iter().for_each(|ws_ref| {
                let socket = ws_ref.clone();
                let message = message.clone();
                let sockets = self.websockets.clone();

                tokio::spawn(async move {
                    if let Err(e) = socket.send(message).await {
                        #[cfg(feature = "logging")]
                        log::debug!("error sending message to websocket: {e}");
                        sockets.write().remove(&socket);
                    }
                });
            })
        }
    }
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
            "api/all_stati" | "api/all_statuses" => {
                let data = self.state_handle.all_stati();
                let api_data: api_types::AllStatiResponse = data.into_iter().map(|(id, values)| {(id, values.into_iter()
                    .map(|(id, status)| {
                        api_types::ElementStatus {
                            id,
                            name: status.name,
                            last_ping: status.last_seen,
                            is_online: status.is_online,
                        }
                    })
                    .collect()
                ) }).collect();
                let text = match rocket::serde::json::to_string(&api_data) {
                    Ok(text) => text,
                    Err(e) => {
                        #[cfg(feature = "logging")]
                        log::error!("Failed to serialize API data: {}", e);
                        return rocket::route::Outcome::Error(rocket::http::Status::InternalServerError)
                    }
                };
                respond_with!(RawJson(text))
            },
            #[cfg(feature = "website-notification-provider-websockets")]
            "ws" => {
                use rocket::futures::FutureExt;
                use rocket::request::FromRequest;
                let handle = tokio::runtime::Handle::current();
                let socket = std::thread::scope(|s| {
                    s.spawn(move || {
                        handle.block_on(ws::WebSocket::from_request(request))
                    }).join().unwrap()
                });
                let socket = match socket {
                    rocket::request::Outcome::Success(socket) => socket,
                    rocket::request::Outcome::Forward(s) => return rocket::route::Outcome::Forward((data, s)),
                    rocket::request::Outcome::Error((e, _)) => return rocket::route::Outcome::Error(e),
                };
                let sockets = self.websockets.clone();
                let channel = socket.channel(move |stream| {
                    async move {
                        sockets.write().insert(websockets::WebSocketRef::new(stream));
                        Ok(())
                    }.boxed()
                });
                match channel.respond_to(request) {
                    Ok(v) => rocket::route::Outcome::Success(v),
                    Err(e) => {
                        #[cfg(feature = "logging")]
                        log::error!("Failed to create WebSocket: {e}");
                        rocket::route::Outcome::Error(e)
                    }
                }
            },
            _ => {
                rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound))
            }
        }
    }
}