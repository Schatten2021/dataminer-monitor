#[cfg(feature = "dataminer_provider")]
mod dataminer;
#[cfg(feature = "e-mails")]
mod email_notifier;
#[cfg(feature = "website")]
mod website;
#[cfg(feature = "server_provider")]
mod server;

#[cfg(feature = "dataminer_provider")]
pub use dataminer::DataMinerInfoSource;
#[cfg(feature = "e-mails")]
pub use email_notifier::EmailNotifier as EmailNotifications;
#[cfg(feature = "website")]
pub use website::Website;
#[cfg(feature = "server_provider")]
pub use server::WebServerInfoProvider;