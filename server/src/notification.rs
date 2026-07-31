use crate::state::AttributeValue;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub component_id: String,
    pub element_id: String,
    pub reason: NotificationReason,
}
impl Notification {
    pub const fn new(component_id: String, element_id: String, reason: NotificationReason) -> Self {
        Self { component_id, element_id, reason, }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NotificationReason {
    /// (new); old is just `!new`
    OnlineStatusChanged(bool),
    AttributeCreated(String, AttributeValue),
    /// (id, old, new)
    AttributeChanged(String, AttributeValue, AttributeValue),
    DeleteAttribute(String, AttributeValue),
    /// (id, online)
    NewElement(String, bool),
}