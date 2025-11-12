mod traits;
mod notifications;
mod state;
mod state_handle;

pub use traits::*;
pub use notifications::{Notification, NotificationReason};
pub use state_handle::StateHandle;

pub struct State(std::sync::Arc<parking_lot::RwLock<state::State>>);
impl State {
    const DEFAULT_CONFIG_PATH: &'static str = "config.toml";
    pub fn register_status_provider<T: StatusProvider>(&self) -> Result<(), ()> {
        self.0.write().register_status_provider::<T>(StateHandle {
            state: self.0.clone(),
            id: T::ID.to_owned(),
            ty: state_handle::ProviderType::Status,
        })
    }
    pub fn register_notification_provider<T: NotificationProvider>(&self) -> Result<(), ()> {
        self.0.write().register_notification_provider::<T>(StateHandle {
            state: self.0.clone(),
            id: T::ID.to_owned(),
            ty: state_handle::ProviderType::Notification,
        })
    }
    pub fn all_stati(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, Status>> {
        self.0.read().get_all_stati()
    }
}
impl Default for State {
    fn default() -> Self {
        Self(std::sync::Arc::new(parking_lot::RwLock::new(state::State::create(Self::DEFAULT_CONFIG_PATH))))
    }
}
impl Clone for State {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-traits", derive(serde::Serialize, serde::Deserialize))]
pub struct Status {
    pub name: String,
    pub is_online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}