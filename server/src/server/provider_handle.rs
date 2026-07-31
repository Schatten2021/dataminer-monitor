use super::Server;
use crate::notification_provider::NotificationProvider;
use crate::state::AttributeValue;
use crate::Component;
use parking_lot::RwLock;
use std::any::TypeId;
use std::sync::Arc;

pub struct ProviderHandle {
    backend: Arc<RwLock<Server>>,
    id: &'static str,
    type_id: TypeId,
}
impl ProviderHandle {
    pub fn attribute_change(self, element_id: &str, attribute_id: &str, value: AttributeValue) {
        self.backend.write()
            .attribute_change(self.id, element_id, attribute_id, value);
    }
    pub(super) fn new<P: Component>(backend: Arc<RwLock<Server>>) -> Self {
        Self { 
            backend, 
            id: P::ID, 
            type_id: TypeId::of::<P>() 
        }
    }
    pub fn add_notification_provider_dependency<P: NotificationProvider>(&self) {
        self.backend.write().add_notification_provider_dependency::<P>(
            Self::new::<P>(self.backend.clone()),
            self.type_id,
        )
    }
    pub fn add_component_dependency<C: Component>(&self) {
        self.backend.write().add_component_dependency::<C>(
            Self::new::<C>(self.backend.clone()),
            self.type_id,
        )
    }
    pub fn component_map<C: Component, F: FnOnce(Option<&C>) -> V, V>(&self, func: F) -> V {
        func(self.backend.read().get_component())
    }
    pub fn component_map_mut<C: Component, F: FnOnce(Option<&mut C>) -> V, V>(&self, func: F) -> V {
        func(self.backend.write().get_component_mut())
    }
    pub fn change_online_state(&self, element_id: &str, status: bool) {
        self.backend.write().online_status_changed(self.id, element_id, status)
    }
    pub fn change_attribute(&self, element_id: &str, attribute_id: &str, value: AttributeValue) {
        self.backend.write().attribute_change(self.id, element_id, attribute_id, value)
    }
}
