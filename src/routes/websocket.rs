use std::hash::Hash;
use std::sync::atomic::AtomicUsize;
use rocket::futures::FutureExt;

#[derive(Clone)]
pub struct WebSocket {
    inner: std::sync::Arc<tokio::sync::Mutex<ws::stream::DuplexStream>>,
    id: usize,
}
mod ws_impl {
    use std::collections::HashSet;
    use std::sync::{Arc, OnceLock};
    use rocket::futures::SinkExt;
    use tokio::sync::{RwLock, Mutex};
    use super::*;
    static SOCKETS: OnceLock<RwLock<HashSet<WebSocket>>> = OnceLock::new();

    impl WebSocket {
        pub fn new(inner: ws::stream::DuplexStream) -> Self {
            static COUNT: AtomicUsize = AtomicUsize::new(0);
            Self {
                inner: Arc::new(Mutex::new(inner)),
                id: COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            }
        }
        fn get_sockets_lock<'a>() -> &'a RwLock<HashSet<WebSocket>> {
            SOCKETS.get_or_init(|| RwLock::new(HashSet::new()))
        }
        pub async fn send(&self, data: api_types::WebSocketMessage) -> Result<(), ws::result::Error> {
            self.inner.lock().await
                .send(ws::Message::Text(rocket::serde::json::to_string(&data).expect("unable to serialize data?!?"))).await
        }
        pub async fn broadcast(data: api_types::WebSocketMessage) {
            Self::get_sockets_lock().read().await.iter().for_each(|socket| {
                let socket = socket.clone();
                let data = data.clone();
                tokio::spawn(async move {
                    if let Err(_) = socket.send(data).await {
                        Self::get_sockets_lock().write().await.remove(&socket);
                    };
                });
            })
        }
        pub async fn add(inner: ws::stream::DuplexStream) {
            let new = Self::new(inner);
            Self::get_sockets_lock().write().await.insert(new);
        }
    }
    impl PartialEq for WebSocket {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl Eq for WebSocket {}
    impl PartialOrd for WebSocket {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.id.partial_cmp(&other.id)
        }
    }
    impl Ord for WebSocket {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }
    impl Hash for WebSocket {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }
}

#[rocket::get("/ws")]
pub async fn websocket<'a>(socket: ws::WebSocket) -> ws::Channel<'a> {
    socket.channel(|stream| {
        async move {
            WebSocket::add(stream).await;
            Ok(())
        }.boxed()
    })
}