use crate::notifications::Notification;
use crate::{state, Status};

#[derive(Clone)]
pub struct StateHandle {
    pub(crate) state: std::sync::Arc<parking_lot::RwLock<state::State>>,
    pub(crate) id: String,
    pub(crate) ty: ProviderType,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ProviderType {
    Status,
    Notification,
}
impl StateHandle {
    pub fn send_notification(&self, notification: Notification) {
        self.state.read().send_notification(&self.id, notification);
    }
    pub fn all_stati(&self) -> std::collections::HashMap<String, std::collections::HashMap<String, Status>> {
        self.state.read().get_all_stati()
    }
    pub fn remove_self(self) {
        let mut lock = self.state.write();
        match self.ty {
            ProviderType::Status => lock.unregister_status_provider(&self.id),
            ProviderType::Notification => lock.unregister_notification_provider(&self.id),
        }
    }
    pub fn add_dependency_notification_provider<T: crate::NotificationProvider>(&self) {
        let state = self.state.clone();
        std::thread::spawn(move || {
            // Note: the return value is ignored here because the job of this function is to ensure that the dependency is met.
            // The function only errors if the given type is already registered, which we don't care about in this situation.
            let _ = state.write().register_notification_provider::<T>(StateHandle {
                state: state.clone(),
                id: T::ID.to_owned(),
                ty: ProviderType::Status,
            });
        });
    }
    pub fn add_dependency_status_provider<T: crate::StatusProvider>(&self) {
        let state = self.state.clone();
        std::thread::spawn(move || {
            // Note: the return value is ignored here because the job of this function is to ensure that the dependency is met.
            // The function only errors if the given type is already registered, which we don't care about in this situation.
            let _ = state.write().register_status_provider::<T>(StateHandle {
                state: state.clone(),
                id: T::ID.to_owned(),
                ty: ProviderType::Status,
            });
        });
    }
}