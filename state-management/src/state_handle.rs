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
}