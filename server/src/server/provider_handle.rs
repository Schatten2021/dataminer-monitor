use super::Server;
use crate::notification_provider::NotificationProvider;
use crate::state::AttributeValue;
use crate::{Component, State};
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
/// Handle to the server for [`Component`]s to use.
///
/// # Note
/// These are customized for each component.
/// If you share these between different components, the resulting notifications _will_ appear to be
/// sent by the wrong component.
pub struct ComponentHandle {
    backend: Arc<RwLock<Server>>,
    id: &'static str,
    type_id: TypeId,
}
impl ComponentHandle {
    pub(super) fn new<P: Component>(backend: Arc<RwLock<Server>>) -> Self {
        Self { 
            backend, 
            id: P::ID, 
            type_id: TypeId::of::<P>() 
        }
    }
    /// Add a [`NotificationProvider`] dependency.
    ///
    /// # Note
    /// There is no check for recursive dependencies. Do not use recursive dependencies.
    pub fn add_notification_provider_dependency<P: NotificationProvider>(&self) {
        self.backend.write().add_notification_provider_dependency::<P>(
            Self::new::<P>(self.backend.clone()),
            self.type_id,
        );
    }
    /// Add a [`Component`] dependency.
    ///
    /// # Note
    /// There is no check for recursive dependencies. Do not use recursive dependencies.
    pub fn add_component_dependency<C: Component>(&self) {
        self.backend.write().add_component_dependency::<C>(
            Self::new::<C>(self.backend.clone()),
            self.type_id,
        );
    }
    /// retrieves a reference to a component from the server and applies the map function to it.
    ///
    /// # Note
    /// This is this way because the actual server is behind an [`Arc`] reference and if we didn't
    /// do it this way there would be lifetime issues.
    // NOTE: I could build a custom struct that houses the reference to the component & the lock.
    //       Might do that in the future.
    pub fn component_map<C: Component, F: FnOnce(Option<&C>) -> V, V>(&self, func: F) -> V {
        func(self.backend.read().get_component())
    }
    /// retrieves a mutable reference to a component from the server and applies the map function to it.
    ///
    /// # Note
    /// This is this way because the actual server is behind an [`Arc`] reference and if we didn't
    /// do it this way there would be lifetime issues.
    // NOTE: I could build a custom struct that houses the reference to the component & the lock.
    //       Might do that in the future.
    pub fn component_map_mut<C: Component, F: FnOnce(Option<&mut C>) -> V, V>(&self, func: F) -> V {
        func(self.backend.write().get_component_mut())
    }
    /// Changes the online state of an element.
    ///
    /// # Note
    /// Please make sure that the state actually changes via [`Self::get_online_state`].
    /// Calling this function without checking the online state first is a lot slower.
    pub fn change_online_state(&self, element_id: &str, status: bool) {
        self.backend.write().online_status_changed(self.id, element_id, status);
    }
    /// Retrieves the online state of an element.
    #[must_use]
    pub fn get_online_state(&self, element_id: &str) -> Option<bool> {
        self.backend.read().get_status(element_id)
    }
    /// Returns a copy of all elements and their states.
    #[must_use]
    pub fn get_states(&self) -> HashMap<String, State> {
        self.backend.read().get_states()
    }
    /// Changes the attribute of an element.
    ///
    /// # Note
    /// Please make sure that the state actually changes via [`Self::get_online_state`].
    /// Calling this function without checking the online state first is a lot slower and sends
    /// unnecessary notifications.
    pub fn change_attribute(&self, element_id: &str, attribute_id: &str, value: AttributeValue) {
        self.backend.write().attribute_change(self.id, element_id, attribute_id, value);
    }
    /// Retrieves the given attribute of an element.
    #[must_use]
    pub fn get_attribute(&self, element_id: &str, attribute_id: &str) -> Option<AttributeValue> {
        self.backend.read().get_attribute(element_id, attribute_id)
    }
    /// Deletes an attribute for an element.
    pub fn delete_attribute(&self, element_id: &str, attribute_id: &str) {
        self.backend.write().delete_attribute(element_id, attribute_id, self.id);
    }
}
