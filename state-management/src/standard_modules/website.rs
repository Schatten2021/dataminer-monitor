use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use rocket::http::uri::fmt::Path;
use rocket::http::uri::Segments;
use rocket::{tokio, Data, Request};
use rocket::response::Responder;
use rocket::route::Outcome;
use parking_lot::RwLock;
use crate::{Notification, NotificationReason, StateHandle};

fn default_static_dir() -> PathBuf {
    PathBuf::from("static/")
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_static_dir")]
    pub static_dir: PathBuf,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            static_dir: default_static_dir(),
        }
    }
}

#[cfg(feature = "frontend-websocket")]
use sockets::WebSocket;
#[cfg(feature = "frontend-websocket")]
mod sockets {
    use super::*;
    #[derive(Clone)]
    pub(super) struct WebSocket {
        id: usize,
        pub(super) inner: Arc<tokio::sync::Mutex<ws::stream::DuplexStream>>,
    }
    impl WebSocket {
        pub(super) fn new(inner: ws::stream::DuplexStream) -> Self {
            static SOCKET_COUNT: AtomicUsize = AtomicUsize::new(0);
            Self {
                id: SOCKET_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                inner: Arc::new(tokio::sync::Mutex::new(inner)),
            }
        }
    }
    impl PartialEq for WebSocket {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl Eq for WebSocket {}
    impl std::hash::Hash for WebSocket {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }
}

pub struct Website {
    config: Config,
    handle: StateHandle,
    #[cfg(feature = "frontend-websocket")]
    websockets: Arc<RwLock<std::collections::HashSet<WebSocket>>>,
}
impl crate::NotificationProvider for Website {
    const ID: &'static str = "website";
    type Config = Config;

    fn new(state: StateHandle, config: Self::Config) -> Self {
        Self {
            config,
            handle: state,
            #[cfg(feature = "frontend-websocket")]
            websockets: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    fn reconfigure(&mut self, config: Self::Config) {
        self.config = config;
    }

    fn send(&self, source_type: String, notification: Notification) {
        #[cfg(feature = "frontend-websocket")]
        {
            use rocket::futures::SinkExt;
            let message = match notification.reason {
                NotificationReason::WentOnline => api_types::WebSocketMessage::MinerStatusChange(api_types::StatusUpdate {
                    type_id: source_type,
                    id: notification.item_id.clone(),
                    new_status: true,
                }),
                NotificationReason::WentOffline => api_types::WebSocketMessage::MinerStatusChange(api_types::StatusUpdate {
                    type_id: source_type,
                    id: notification.item_id.clone(),
                    new_status: false,
                }),
                NotificationReason::Seen => api_types::WebSocketMessage::MinerPing {
                    type_id: source_type,
                    miner_id: notification.item_id,
                }
            };
            let message_text = rocket::serde::json::to_string(&message).unwrap();
            let message = ws::Message::Text(message_text);
            self.websockets.read().iter().for_each(|websocket| {
                let socket = websocket.clone();
                let message = message.clone();
                let sockets_ref = self.websockets.clone();

                tokio::spawn(async move {
                    if let Err(_) = socket.inner.lock().await.send(message).await {
                        sockets_ref.write().remove(&socket);
                    }
                });
            });
        }
    }
    fn handle_request<'r, 'l>(&self, path: Segments<Path>, request: &'r Request<'l>, data: Data<'r>) -> Outcome<'r> {
        macro_rules! respond_with {
            ($what:ident($val:expr)) => {
                {
                    match rocket::response::content::$what($val).respond_to(request) {
                        Ok(response) => Outcome::Success(response),
                        Err(e) => Outcome::Error(e),
                    }
                }
            };
        }
        let path = &*path.collect::<Vec<_>>().join("/");
        match path {
            // route "/"
            "" | "index.html" | "static/index.html" => respond_with!(RawHtml(self.index())),
            "static/style.css" => respond_with!(RawCss(self.style())),
            "static/wasm/frontend.js" => respond_with!(RawJavaScript(self.wasm_js())),
            "static/wasm/frontend_bg.wasm" => {
                #[derive(rocket::Responder)]
                #[response(status = 200, content_type = "application/wasm")]
                struct Responder(Vec<u8>);
                match Responder(self.wasm_bin()).respond_to(request) {
                    Ok(response) => Outcome::Success(response),
                    Err(e) => Outcome::Error(e),
                }
            },
            "api/all_stati" | "api/all_statuses" => {
                let data = self.handle.get_all_stati();
                let api_data: api_types::AllStatiResponse = data.into_iter().map(|(id, values)| {
                    (id, values.into_iter().map(|(id, status)| {
                        api_types::ElementStatus {
                            id,
                            name: status.name,
                            last_ping: status.last_seen.as_ref().map(chrono::DateTime::to_utc),
                            is_online: status.is_online,
                        }
                    }).collect::<Vec<_>>())
                }).collect::<HashMap<_, _>>();
                let text = match rocket::serde::json::to_string(&api_data) {
                    Ok(t) => t,
                    Err(_e) => {
                        return Outcome::Error(rocket::http::Status::InternalServerError);
                    }
                };
                respond_with!(RawJson(text))
            },
            #[cfg(feature = "frontend-websocket")]
            "ws" => {
                use rocket::futures::FutureExt;
                use rocket::request::FromRequest;
                let rocket::request::Outcome::Success(socket) = pollster::block_on(ws::WebSocket::from_request(request)) else {
                    return Outcome::Error(rocket::http::Status::InternalServerError);
                };
                let sockets = self.websockets.clone();
                let channel = socket.channel(move |stream| {
                    async move {
                        sockets.write().insert(WebSocket::new(stream));
                        Ok(())
                    }.boxed()
                });
                match channel.respond_to(request) {
                    Ok(v) => Outcome::Success(v),
                    Err(e) => Outcome::Error(e),
                }
            }
            _ => Outcome::Forward((data, rocket::http::Status::NotFound)),
        }
    }
}
macro_rules! include_file {
    ($ident:ident: $target_ty:ty => $read_func:ident($path:literal) | $include:ident!($default_path:literal)$(.$post_process_func:ident)*) => {
        fn $ident(&self) -> $target_ty {
            #[allow(unused_mut)]
            let mut content: $target_ty = $include!($default_path)$(.$post_process_func())*;
            {
                if let Ok(value) = std::fs::$read_func(self.config.static_dir.join($path)) {
                    content = value;
                }
            }
            content
        }
    };
    (string $func:ident => $path:literal | $default_path:literal) => {
        include_file!($func: String => read_to_string($path) | include_str!($default_path).to_string);
    };
}
impl Website {
    include_file!(string index => "index.html" | "../../../static/index.html");
    include_file!(string style => "style.css" | "../../../static/style.css");
    include_file!(string wasm_js => "wasm/frontend.js" | "../../../static/wasm/frontend.js");
    include_file!(wasm_bin: Vec<u8> => read("wasm/frontend_bg.wasm") | include_bytes!("../../../static/wasm/frontend_bg.wasm").to_vec);
}