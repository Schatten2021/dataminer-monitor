use super::Server;
use crate::notification_provider::NotificationProvider;
use crate::{Component, ComponentHandle};
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use axum::extract::Request;
use crate::state::State;

#[derive(Clone)]
pub struct ServerHandle(Arc<RwLock<Server>>);
impl ServerHandle {
    // register provider
    pub fn new(config_path: PathBuf) -> Self {
        Self(Arc::new(RwLock::new(Server::new(config_path))))
    }
    pub fn add_component<C: Component>(&self) -> &Self {
        self.0.write().add_component::<C>(self.provider_handle::<C>());
        self
    }
    pub fn remove_component<C: Component>(&self) -> &Self {
        self.0.write().remove_component(TypeId::of::<C>());
        self
    }
    pub fn add_notification_provider<P: NotificationProvider>(&self) -> &Self {
        self.0.write().add_notification_provider::<P>(self.provider_handle::<P>());
        self
    }
    pub fn reload_config(&self) -> &Self {
        self.0.write().reload_config();
        self
    }
    pub fn component_map<C: Component, F: FnOnce(Option<&C>) -> V, V>(&self, func: F) -> V {
        func(self.0.read().get_component())
    }
    pub fn component_map_mut<C: Component, F: FnOnce(Option<&mut C>) -> V, V>(&self, func: F) -> V {
        func(self.0.write().get_component_mut())
    }
    fn provider_handle<P: Component>(&self) -> ComponentHandle {
        ComponentHandle::new::<P>(self.0.clone())
    }
    pub fn get_states(&self) -> HashMap<String, State> {
        self.0.read().get_states()
    }
}
impl axum::handler::Handler<(), ()> for ServerHandle {
    type Future = Pin<Box<dyn Future<Output=axum::response::Response> + Send + 'static>>;

    fn call(self, req: Request, (): ()) -> Self::Future {
        match self.0.read().try_handle_request(req) {
            Ok(v) => v,
            Err(r) => Box::pin(std::future::ready({
                info!("unable to handle request to {}", r.uri());
                axum::response::Response::builder()
                    .status(404)
                    .body(axum::body::Body::new(r#"<script> window.socket = new WebSocket("ws://127.0.0.1:8000/api/ws");</script>"#.to_string()))
                    .unwrap()
            }))
        }
    }
}