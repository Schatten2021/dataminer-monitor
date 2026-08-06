//! utilities to enable easier filtering of messages for [`server::StatusProvider`].

use server::{Notification, NotificationReason};
use std::hash::Hash;

const fn always() -> bool { true }

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
    pub whitelist: Vec<ItemType>,
    #[serde(alias="deny", alias="denied", alias="denies", alias="disable", alias="disabled", alias="blacklisted",
        alias="disallow", alias="disallowed", alias="disallows")]
    #[serde(default="Vec::new")]
    /// Blacklist of things to block.
    pub blacklist: Vec<ItemType>,

    /// Whether to accept values per default or to reject them.
    ///
    /// Changes the behavior of the filter.
    #[serde(default)]
    #[serde(alias="default", alias="mode")]
    pub priority: FilterPriority,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// What of the filter to prioritize.
pub enum FilterPriority {
    /// Prioritizes the whitelist.
    ///
    /// This makes it so that the [`SingleFilter`] accepts values by default and only rejects values
    /// if they are in the blacklist.
    #[serde(alias="allow", alias="accept", alias="explicit-blacklist", alias="explicit_blacklist")]
    #[default]
    Whitelist,
    /// Prioritizes the blacklist
    ///
    /// This makes it so that the [`SingleFilter`] accepts values by default and only accepts values
    /// if they are in the whitelist.
    #[serde(alias="disallow", alias="deny", alias="explicit-whitelist", alias="explicit_whitelist")]
    Blacklist
}
impl<T> Default for SingleFilter<T> {
    fn default() -> Self {
        Self {
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            priority: FilterPriority::Whitelist,
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
    OnlineStateChange(OnlineStateChange)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all="snake_case")]
/// changes to the online state.
pub enum OnlineStateChange {
    /// matches any online state change
    Any,

    /// when the server went online
    #[serde(alias="up")]
    Online,

    /// when the server went offline
    #[serde(alias="down")]
    Offline,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
/// Matches when attributes of an element change.
pub struct AttributeChange {
    #[serde(flatten)]
    id: AttributeIdMatcher,

    #[serde(default)]
    /// The actual element being matched.
    event: AttributeEvent,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Default)]
/// Matches an AttributeId.
pub struct AttributeIdMatcher {
    /// The id of the attribute.
    id: Option<String>,

    #[serde(default="always")]
    /// whether to match the id exactly (no children)
    exact: bool,
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
        match self.priority {
            FilterPriority::Whitelist => self.whitelisted(input) || !self.blacklisted(input),
            FilterPriority::Blacklist => self.whitelisted(input) && !self.blacklisted(input)
        }
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
impl Filtering<bool> for OnlineStateChange {
    fn matches(&self, value: &bool) -> bool {
        matches!((self, value),
            (Self::Any, _) |
            (Self::Online, true) |
            (Self::Offline, false)
        )
    }
}
impl Filtering<NotificationReason> for StateChange {
    fn matches(&self, reason: &NotificationReason) -> bool {
        match self {
            Self::OnlineStateChange(filter)=> matches!(reason,
                NotificationReason::OnlineStatusChanged(status) |
                NotificationReason::NewElement(status)
                if filter.matches(status)),
            Self::CreateEntity => matches!(reason, NotificationReason::NewElement(_)),
            Self::AttributeChange(change) => match reason {
                NotificationReason::AttributeCreated(id, _) |
                NotificationReason::AttributeChanged(id, _, _) |
                NotificationReason::DeleteAttribute(id, _) => change.event.matches(reason) && change.id.matches(id),
                _ => false,
            }
        }
    }
}
impl Filtering<String> for AttributeIdMatcher {
    fn matches(&self, value: &String) -> bool {
        if self.exact {
            self.id.as_ref().is_none_or(|filter| filter == value)
        } else {
            self.id.as_ref()
                .is_none_or(|filter|
                    value.starts_with(filter) &&
                        value.get(filter.len()..)
                            .is_none_or(|remaining| remaining.starts_with('.'))
                )
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