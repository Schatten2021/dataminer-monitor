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
pub enum Filter {
    #[serde(alias="white list", alias="whitelist", alias="allow", alias="send", alias="accept", alias="accepted")]
    WhiteList(HashSet<state_management::NotificationReason>),
    #[serde(alias="blacklist", alias="black list", alias="deny", alias="reject", alias="rejected", alias="filter")]
    BlackList(HashSet<state_management::NotificationReason>),
}
impl Default for Filter {
    fn default() -> Self {
        Filter::BlackList(HashSet::from([state_management::NotificationReason::Seen]))
    }
}
impl Filter {
    pub fn allows(&self, reason: &state_management::NotificationReason) -> bool {
        match self {
            Filter::WhiteList(whitelist) => whitelist.contains(reason),
            Filter::BlackList(blacklist) => !blacklist.contains(reason),
        }
    }
}