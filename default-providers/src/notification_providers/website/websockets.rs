use std::sync::atomic::AtomicU64;
use rocket::futures::SinkExt;

pub struct WebSocketRef {
    id: u64,
    inner: std::sync::Arc<tokio::sync::Mutex<ws::stream::DuplexStream>>,
}
impl Clone for WebSocketRef {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}
impl WebSocketRef {
    pub fn new(inner: ws::stream::DuplexStream) -> Self {
        static SOCKET_COUNT: AtomicU64 = AtomicU64::new(0);
        Self {
            id: SOCKET_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(inner)),
        }
    }
    pub async fn send(&self, message: String) -> Result<(), ws::result::Error>{
        let mut lock = self.inner.lock().await;
        lock.send(ws::Message::Text(message)).await
    }
}
impl PartialEq for WebSocketRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for WebSocketRef {}
impl std::hash::Hash for WebSocketRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
