//! utilities to enable easier filtering of messages for [`server::StatusProvider`].

use server::{Notification, NotificationReason};
use std::hash::Hash;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
/// A fully configurable filter.
///
/// Allows filtering for different components, entities & messages, down to the id of a changed attribute.
pub struct Filter {
    #[serde(alias="components", alias="component-id", alias="component_id", alias="source", alias="source-id", alias="source_id")]
    #[serde(default)]
    /// [`SingleFilter`] filtering the id of the component that caused the [`Notification`].
    pub component: SingleFilter<String>,

    #[serde(default)]
    #[serde(alias="entities", alias="entity-id", alias="entity_id",
        alias="element", alias="elements", alias="element-id", alias="element_id")]
    /// [`SingleFilter`] filtering the id of the target entity of the [`Notification`].
    pub entity: SingleFilter<String>,

    #[serde(alias="state-change",
        alias="state", alias="states",
        alias="status", alias="statuses", alias="stati",
        alias="change", alias="changes")]
    /// [`SingleFilter`] filtering the state changes.
    pub state_changes: SingleFilter<StateChange>,
}
impl Filter {
    /// whether the filter allows the given message.
    #[must_use]
    pub fn allows(&self, message: &Notification) -> bool {
        self.component.allows(&message.component_id) &&
            self.entity.allows(&message.element_id) &&
            self.state_changes.allows(&message.reason)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// Filter filtering a single type of item.
pub struct SingleFilter<ItemType> {
    #[serde(alias="allow", alias="allowed", alias="enable", alias="enabled", alias="whitelisted",
        alias="accept", alias="accepts", alias="accepted")]
    #[serde(default="Vec::new")]
    /// Whitelist of things to specifically allow.
    ///
    /// # Note
    /// Whitelist overrules [`blacklist`]
    whitelist: Vec<ItemType>,
    #[serde(alias="deny", alias="denied", alias="denies", alias="disable", alias="disabled", alias="blacklisted",
        alias="disallow", alias="disallowed", alias="disallows")]
    #[serde(default="Vec::new")]
    /// Blacklist of things to block.
    ///
    /// # Note
    /// [`whitelist`] overrules the Blacklist.
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
#[serde(rename_all="snake_case")]
/// Identifies a change in state for filtering.
pub enum StateChange {
    #[serde(alias="create-entity", alias="create")]
    /// Matches the creation of an entity (no matter the new state. Use [`Self::OnlineStateChange`] for that.)
    CreateEntity,

    #[serde(alias="attribute")]
    /// Matches changes to the attributes of an element. See [`AttributeChange`] for more infos.
    AttributeChange(AttributeChange),

    #[serde(alias="online", alias="online-state", alias="online_state")]
    /// matches changes to the online state.
    ///
    /// If the boolean is unset it matches both `true` and `false`.
    OnlineStateChange(Option<bool>)
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
/// Matches when attributes of an element change.
pub struct AttributeChange {
    /// The id of the attribute.
    id: Option<String>,
    #[serde(default)]
    /// The actual element being matched.
    event: AttributeEvent,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all="snake_case")]
/// Events that can happen to an attribute.
pub enum AttributeEvent {
    #[default]
    /// Match any change to the attribute.
    Any,
    /// Match the creation of a new attribute.
    Create,
    /// Match the change of an attribute.
    Change,
    /// Match the deletion of an attribute.
    Delete,
}
impl<Item> SingleFilter<Item> {
    /// checks whether the filter allows the given input.
    pub fn allows<V>(&self, input: &V) -> bool
    where Item: Filtering<V> {
        self.whitelisted(input) || !self.blacklisted(input)
    }
    /// checks whether the input is whitelisted
    pub fn whitelisted<V>(&self, input: &V) -> bool
    where Item: Filtering<V> {
        self.whitelist.iter().any(|f| f.matches(input))
    }
    /// checks whether the input is blacklisted
    pub fn blacklisted<V>(&self, input: &V) -> bool
    where Item: Filtering<V> {
        self.blacklist.iter().any(|f| f.matches(input))
    }
    
}
/// Helper trait for usage with [`SingleFilter`].
///
/// Implemented by default for types that implement [`Eq`].
pub trait Filtering<Input> {
    /// Check whether the value matches given the configuration of `self`.
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
                NotificationReason::NewElement(_)),
            Self::OnlineStateChange(Some(status_filter)) => matches!(reason,
                NotificationReason::OnlineStatusChanged(new) |
                NotificationReason::NewElement(new)
                if new == status_filter),
            Self::CreateEntity => matches!(reason, NotificationReason::NewElement(_)),
            Self::AttributeChange(change) => match reason {
                NotificationReason::AttributeCreated(id, _) |
                NotificationReason::AttributeChanged(id, _, _) |
                NotificationReason::DeleteAttribute(id, _) => change.event.matches(reason) &&
                    change.id.as_ref().is_none_or(|v| v == id),
                _ => false,
            }
        }
    }
}
impl Filtering<NotificationReason> for AttributeEvent {
    fn matches(&self, value: &NotificationReason) -> bool {
        !matches!(value, NotificationReason::NewElement(_) | NotificationReason::OnlineStatusChanged(_)) &&
            matches!((self, value),
                (Self::Any, _) |
                (Self::Create, NotificationReason::AttributeCreated(_, _)) |
                (Self::Change, NotificationReason::AttributeChanged(_, _, _)) |
                (Self::Delete, NotificationReason::DeleteAttribute(_, _))
            )
    }
}