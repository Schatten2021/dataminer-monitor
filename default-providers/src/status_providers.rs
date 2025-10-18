#[cfg(feature = "dataminer-status-provider")]
mod dataminer;
#[cfg(feature = "website-status-provider")]
mod website;

#[cfg(feature = "dataminer-status-provider")]
pub use dataminer::DataminerStatusProvider;
#[cfg(feature = "website-status-provider")]
pub use website::ServerStatusProvider;