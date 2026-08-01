//! Library containing default components for my [Status server](server).
//!
//! Contains
//! - [`WebsiteStatuse`]: component for keeping track of the status of websites
//! - [`filters`]: utilities for configuring filters for [`server::NotificationProvider`]
//! - [`Api`]: An API for interacting with the [Status server](server)
//! - [`Frontend`]: A web frontend for easily checking the state of the server and elements.
//! - [`DataminerStatus`]: Component for keeping track of the status of dataminers.
//! - [`MinecraftStatus`]: Component for keeping track of the status of minecraft servers.
//! - [`EmailNotificationProvider`]: [`server::NotificationProvider`] for sending E-Mail notifications.
//! - [`NtfyNotificationProvider`]: [`server::NotificationProvider`] for sending Push-Notifications via [NTFY](https://ntfy.sh/)
 

#![cfg_attr(not(debug_assertions), deny(missing_docs))]
#![cfg_attr(debug_assertions, warn(missing_docs))]
#![warn(clippy::pedantic)]
#![warn(clippy::complexity, clippy::suspicious, clippy::perf, clippy::style, clippy::allow_attributes_without_reason)]
#![allow(
    clippy::needless_continue,
    reason = "adding a `continue` often makes the code easier to read."
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    reason = "don't want these lints."
)]
#![cfg_attr(not(debug_assertions), deny(clippy::undocumented_unsafe_blocks))]
#![cfg_attr(debug_assertions, warn(clippy::undocumented_unsafe_blocks))]

#[macro_use]
extern crate tracing;

pub mod filters;
mod website;
mod api;
mod frontend;
mod dataminer;
mod minecraft;
mod email;
mod ntfy;

pub use website::WebsiteStatuse;
pub use api::Api;
pub use frontend::Frontend;
pub use dataminer::DataminerStatus;
pub use minecraft::MinecraftStatus;
pub use email::EmailNotificationProvider;
pub use ntfy::NtfyNotificationProvider;