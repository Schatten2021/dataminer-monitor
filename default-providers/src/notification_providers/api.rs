use rocket::response::Responder;
use state_management::{Notification, StateHandle};

mod websockets;

fn default_route() -> String { "api/".to_string() }
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    #[serde(default = "default_route")]
    pub route: String,
}
impl Default for Config {
    fn default() -> Self {
        Self { route: default_route() }
    }
}

pub struct ApiNotificationProvider {
    config: Config,
    state_handle: state_management::StateHandle,
    websockets: std::sync::Arc<parking_lot::RwLock<std::collections::HashSet<websockets::WebSocketRef>>>,
}
impl state_management::NotificationProvider for ApiNotificationProvider {
    const ID: &'static str = "api";
    type Config = Config;

    fn new(state: StateHandle, config: Self::Config) -> Self {
        Self {
            state_handle: state,
            config,
            websockets: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashSet::new())),
        }
    }

    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
    }

    fn send(&self, source_id: String, notification: Notification) {
        #[cfg(feature = "api-notification-provider-websockets")]
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
        let path = path.collect::<Vec<_>>().join("/");
        if !path.starts_with(&self.config.route) {
            return rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound));
        }
        let mut path = &path[self.config.route.len()..];
        if path.starts_with("/") { path = &path[1..]; }
        match path {
            "all_stati" | "all_statuses" => {
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
                let response = rocket::serde::json::Json(api_data);
                match response.respond_to(request) {
                    Ok(response) => rocket::route::Outcome::Success(response),
                    Err(e) => {
                        #[cfg(feature = "logging")]
                        log::error!("error responding: {e}");
                        rocket::route::Outcome::Error(rocket::http::Status::InternalServerError)
                    }
                }
            },
            #[cfg(feature = "api-notification-provider-websockets")]
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
            _ => rocket::route::Outcome::Forward((data, rocket::http::Status::NotFound)),
        }
    }
}