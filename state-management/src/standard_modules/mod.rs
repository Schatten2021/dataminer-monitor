#[cfg(feature = "dataminer-status-source")]
mod dataminer;
#[cfg(feature = "e-mail-notifications")]
mod email_notifier;
#[cfg(feature = "frontend-website")]
mod website;
#[cfg(feature = "server-status-source")]
mod server;

#[cfg(feature = "dataminer-status-source")]
pub use dataminer::DataMinerInfoSource;
#[cfg(feature = "e-mail-notifications")]
pub use email_notifier::EmailNotifier as EmailNotifications;
#[cfg(feature = "frontend-website")]
pub use website::Website;
#[cfg(feature = "server-status-source")]
pub use server::WebServerInfoProvider;