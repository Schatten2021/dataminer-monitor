#[cfg(feature = "email-notification-provider")]
mod email;
#[cfg(feature = "website-notification-provider")]
mod website;
#[cfg(feature = "api-notification-provider")]
mod api;
#[cfg(feature = "ntfy-notification-provider")]
mod ntfy;

use std::collections::HashSet;
#[cfg(feature = "email-notification-provider")]
pub use email::EmailNotificationProvider;
#[cfg(feature = "website-notification-provider")]
pub use website::WebsiteNotificationProvider;
#[cfg(feature = "api-notification-provider")]
pub use api::ApiNotificationProvider;
#[cfg(feature = "ntfy-notification-provider")]
pub use ntfy::NtfyNotificationProvider;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Filter {
    #[serde(flatten)]
    reason_filter: ReasonFilter,
    #[serde(flatten)]
    id_filter: IdFilter,
    #[serde(flatten)]
    type_filter: TypeFilter,
}
macro_rules! filter {
    ($name:ident<$ty:ty> {
        whitelist: $whitelist_main_name:literal $(,$whitelist_aliases:literal)*$(,)?
        blacklist: $blacklist_main_name:literal $(,$blacklist_aliases:literal)*$(,)?
    }) => {
        #[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug)]
        pub enum $name {
            #[serde(rename=$whitelist_main_name)]
            #[serde($(alias=$whitelist_aliases),*)]
            WhiteList(HashSet<$ty>),
            #[serde(rename=$blacklist_main_name)]
            #[serde($(alias=$blacklist_aliases),*)]
            BlackList(HashSet<$ty>)
        }
        impl $name {
            fn allows(&self, val: &$ty) -> bool {
                match self {
                    Self::WhiteList(whitelist) => whitelist.contains(val),
                    Self::BlackList(blacklist) => !blacklist.contains(val),
                }
            }
        }
        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::BlackList(HashSet::new())
            }
        }
    };
}
filter!(ReasonFilter<state_management::NotificationReason> {
    whitelist: "whitelist", "white list", "reason whitelist",
    "reason_whitelist", "reason-whitelist", "whitelist-reasons", "whitelist-reason", "whitelist_reasons", "whitelist_reason",
    "allow", "allow-reasons", "allowed-reasons", "allow_reasons", "allowed_reasons",
    blacklist: "blacklist", "black list", "reason blacklist",
    "reason_blacklist", "reason-blacklist", "blacklist-reasons", "blacklist-reason", "blacklist_reasons", "blacklist_reason",
    "reject", "reject-reasons", "rejected-reasons", "reject_reasons", "rejected_reasons",
    "deny", "deny-reasons", "denied-reasons", "deny_reasons", "denied_reasons"
});
filter!(IdFilter<String> {
    whitelist: "id-whitelist", "id_whitelist", "whitelisted-ids", "whitelisted_ids",
    "allowed-ids", "allow-ids", "allowed_ids", "allow_ids",
    blacklist: "id-blacklist", "id_blacklist", "blacklisted-ids", "blacklisted_ids",
    "rejected-ids", "reject-ids", "rejected_ids", "reject_ids"
});
filter!(TypeFilter<String> {
    whitelist: "type-whitelist", "type_whitelist", "whitelisted_types", "whitelisted-types",
    "allowed-types", "allow-types", "allowed_types", "allow_types",
    blacklist: "type-blacklist", "type_blacklist", "blacklisted-types", "blacklisted_types",
    "rejected-types", "reject-types", "rejected_types", "reject_types",
});
impl Default for Filter {
    fn default() -> Self {
        Self {
            reason_filter: ReasonFilter::BlackList(HashSet::from([state_management::NotificationReason::Seen])),
            id_filter: IdFilter::default(),
            type_filter: TypeFilter::default(),
        }
    }
}
impl Filter {
    pub fn allows(&self, source_type_id: &String, notification: &state_management::Notification) -> bool {
        self.reason_filter.allows(&notification.reason) &&
            self.type_filter.allows(source_type_id) &&
            self.id_filter.allows(&notification.item_id)
    }
}
