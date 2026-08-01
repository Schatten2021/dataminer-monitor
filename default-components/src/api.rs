use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use axum::body::Body;
use axum::response::IntoResponse;
use tokio::sync::{Mutex, RwLock};
use utils::Never;
use api_types::{ApiResponse, ServerError};
use server::{ComponentHandle, Notification};

fn default_path() -> String { "api/".to_string() }


#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default="default_path")]
    path: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            path: default_path(),
        }
    }
}

struct Socket {
    ws: Mutex<axum::extract::ws::WebSocket>,
    online: AtomicBool,
}

pub struct Api {
    state: ComponentHandle,
    websockets: Arc<RwLock<Vec<Socket>>>,
    config: Config,
}

fn should_handle_path(mut path: &str, mut prefix: &str) -> bool {
    path = path.trim_start_matches('/');
    prefix = prefix.trim_matches('/');
    if !path.starts_with(prefix) {
        trace!("not a request to the API");
        return false;
    }
    path = &path[prefix.len()..];
    trace!("got api request to endpoint \"{path}\"");
    matches!(
        path,
        "" | "/" |
        "/current" |
        "/ws" | "/websocket" | "/socket"
    )
}

impl server::Component for Api {
    const ID: &'static str = "";
    type Config = Config;
    type ConfigError = Never;

    fn init(server: ComponentHandle, config: Self::Config) -> Result<Self, Self::ConfigError> {
        let websockets = Arc::new(RwLock::new(Vec::<Socket>::new()));
        let mut ticker = tokio::time::interval(Duration::from_mins(30));
        let ws = websockets.clone();
        tokio::spawn(async move {
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let mut lock = ws.write().await;
                let mut vec = Vec::new();
                core::mem::swap(&mut *lock, &mut vec);
                *lock = vec.into_iter()
                    .filter(|socket| socket.online.load(Ordering::Relaxed))
                    .collect();
            }
        });
        Ok(Self {
            state: server,
            websockets,
            config,
        })
    }

    fn reconfigure(&mut self, config: Self::Config) -> Result<(), Self::ConfigError> {
        self.config = config;
        Ok(())
    }

    fn try_handle(&self, request: axum::extract::Request) -> Result<server::RequestHandle, axum::extract::Request> {
        macro_rules! json {
            ($code:expr, $val:expr) => {
                match ::serde_json::to_string(&$val) {
                    Ok(v) => ($code, v),
                    Err(e) => {
                        error!("couldn't JSON-serialize response: {e}");
                        match ::serde_json::to_string(&ApiResponse::<(), ()>::ServerError(ServerError {
                            id: "json.serialize".to_string(),
                            message: e.to_string(),
                        })) {
                            Ok(v) => (500, v),
                            Err(e) => {
                                error!("couldn't JSON serialize JSON serialization error?!?!? ({e})");
                                (200, "{}".to_string())
                            }
                        }
                    }
                }
            };
        }
        macro_rules! ok {
            ($val:expr) => {
                json!(200, ::api_types::ApiResponse::<_, ()>::Ok($val))
            };
        }
        macro_rules! err {
            ($code:literal, $val:expr) => {
                json!($code, ::api_types::ApiResponse::<(), _>::ClientError($val))
            };
        }
        macro_rules! exception {
            ($id:literal, $msg:expr) => {
                json!(500, ::api_types::ApiResponse::<(), ()>::ServerError(::api_types::ServerError {
                    id: $id.to_string(),
                    message: $msg.to_string()
                }))
            };
        }
        if !should_handle_path(request.uri().path(), &self.config.path) {
            return Err(request)
        }
        let path_prefix_len = self.config.path.len();
        let state = self.state.clone();
        let websockets = self.websockets.clone();
        Ok(Box::pin(async move {
            let path = &request.uri().path()[path_prefix_len..];
            let _ = request;
            let (code, json) = match path {
                "/" => ok!("Welcome to the API!"),
                "/current" => ok!(state.get_states()),
                // TODO: add routes for requesting selected elements/stati/etc.
                "/ws" | "/websocket" | "/socket" => {
                    use axum::extract::{
                        ws::WebSocketUpgrade,
                        FromRequest,
                    };
                    return match WebSocketUpgrade::from_request(request, &()).await {
                        Ok(upgrade) => {
                            upgrade.on_upgrade(|socket| async move {
                                let socket = Socket {
                                    ws: Mutex::new(socket),
                                    online: AtomicBool::new(true),
                                };
                                websockets.write().await.push(socket);
                            })
                        }
                        Err(e) => e.into_response(),
                    };
                }
                _ => {
                    error!("route set to handle but no handle registered!");
                    exception!("unhandled.route", "Route marked as handled without a handle registered!")
                }
            };
            axum::response::Response::builder()
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET")
                .header("Access-Control-Allow-Headers", "*")
                .status(code)
                .body(Body::new(json))
                .expect("some argument failed to parse?")
        }))
    }
}
impl server::NotificationProvider for Api {
    fn notify(&self, notification: Notification) {
        use axum::extract::ws::{Message, Utf8Bytes};
        let sockets = self.websockets.clone();
        let message: Utf8Bytes = match serde_json::to_string(&notification) {
            Ok(v) => v,
            Err(e) => {
                error!("couldn't serialize notification: {e}");
                return;
            }
        }.into();
        tokio::spawn(async move {
            let sockets = sockets;
            for socket in sockets.read().await.iter() {
                if socket.online.load(Ordering::Relaxed) {
                    let msg = message.clone();
                    trace!("sending {message} to websockets");
                    if let Err(e) = socket.ws.lock().await
                        .send(Message::Text(msg)).await {
                        error!("error sending to websocket: {e}");
                        socket.online.store(false, Ordering::Relaxed);
                    }
                }
            }
        });
    }
}
