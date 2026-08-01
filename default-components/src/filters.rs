use server::{Notification, NotificationReason};
use std::hash::Hash;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct Filter {
    #[serde(alias="components", alias="component-id", alias="component_id", alias="source", alias="source-id", alias="source_id")]
    #[serde(default)]
    component: SingleFilter<String>,

    #[serde(default)]
    #[serde(alias="entities", alias="entity-id", alias="entity_id",
        alias="element", alias="elements", alias="element-id", alias="element_id")]
    entity: SingleFilter<String>,

    #[serde(default)]
    #[serde(alias="state-change",
        alias="state", alias="states",
        alias="status", alias="statuses", alias="stati",
        alias="change", alias="changes")]
    state_changes: SingleFilter<StateChange>,
}
impl Filter {
    pub fn allows(&self, message: &Notification) -> bool {
        self.component.allows(&message.component_id) &&
            self.entity.allows(&message.element_id) &&
            self.state_changes.allows(&message.reason)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SingleFilter<ItemType> {
    #[serde(alias="allow", alias="allowed", alias="enable", alias="enabled", alias="whitelisted")]
    whitelist: Vec<ItemType>,
    #[serde(alias="deny", alias="denied", alias="disable", alias="disabled", alias="blacklisted")]
    blacklist: Vec<ItemType>,
}
impl<T> Default for SingleFilter<T> {
    fn default() -> Self {
        Self {
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum StateChange {
    #[serde(alias="create-entity", alias="create")]
    CreateEntity,
    #[serde(alias="attribute")]
    AttributeChange(AttributeChange),
    #[serde(alias="online", alias="online-state", alias="online_state")]
    OnlineStateChange(Option<bool>)
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum SingleEvent {
    #[serde(alias="create")]
    CreateEntity,
    #[serde(alias="create-attribute", alias="create_attribute", alias="create.attribute")]
    CreateAttribute,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct AttributeChange {
    id: Option<String>,
    #[serde(default)]
    event: AttributeEvent,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Default)]
pub enum AttributeEvent {
    #[default]
    Any,
    Create,
    Change,
    Delete,
}
impl<Item> SingleFilter<Item> {
    pub fn allows<V>(&self, input: &V) -> bool
    where Item: Filtering<V> {
        self.whitelist.iter().any(|f| f.matches(input)) ||
            !self.blacklist.iter().any(|f| f.matches(input))
    }
}
pub trait Filtering<Input> {
    fn matches(&self, value: &Input) -> bool;
}
impl<T: Eq> Filtering<T> for T {
    fn matches(&self, value: &T) -> bool {
        self == value
    }
}
impl Filtering<NotificationReason> for StateChange {
    fn matches(&self, reason: &NotificationReason) -> bool {
        match self {
            Self::OnlineStateChange(None) => matches!(reason,
                NotificationReason::OnlineStatusChanged(_) |
                NotificationReason::NewElement(_, _)),
            Self::OnlineStateChange(Some(status_filter)) => matches!(reason,
                NotificationReason::OnlineStatusChanged(new) |
                NotificationReason::NewElement(_, new)
                if new == status_filter),
            Self::CreateEntity => matches!(reason, NotificationReason::NewElement(_, _)),
            Self::AttributeChange(change) => match reason {
                NotificationReason::AttributeCreated(id, _) |
                NotificationReason::AttributeChanged(id, _, _) |
                NotificationReason::DeleteAttribute(id, _) => change.event.matches(reason) &&
                    change.id.as_ref()
                        .map(|v| v == id)
                        .unwrap_or(true),
                _ => false,
            }
        }
    }
}
impl AttributeEvent {
    pub fn matches(&self, reason: &NotificationReason) -> bool {
        matches!((self, reason),
            (Self::Any, _) |
            (Self::Create, NotificationReason::AttributeCreated(_, _)) |
            (Self::Change, NotificationReason::AttributeChanged(_, _, _)) |
            (Self::Delete, NotificationReason::DeleteAttribute(_, _))
        )
    }
}