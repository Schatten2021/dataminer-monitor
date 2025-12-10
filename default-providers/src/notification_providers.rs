#[cfg(feature = "email-notification-provider")]
mod email;
#[cfg(feature = "website-notification-provider")]
mod website;
#[cfg(feature = "api-notification-provider")]
mod api;
#[cfg(feature = "ntfy-notification-provider")]
mod ntfy;

#[cfg(feature = "email-notification-provider")]
pub use email::EmailNotificationProvider;
#[cfg(feature = "website-notification-provider")]
pub use website::WebsiteNotificationProvider;
#[cfg(feature = "api-notification-provider")]
pub use api::ApiNotificationProvider;
#[cfg(feature = "ntfy-notification-provider")]
pub use ntfy::NtfyNotificationProvider;