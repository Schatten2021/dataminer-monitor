pub mod notification_providers;
pub mod status_providers;
macro_rules! log {
    ($method:ident status $name:literal: $msg:literal $(,$args:expr)*) => {#[cfg(feature = "logging")] ::log::$method!(concat!("STATUS: \"", $name, "\"", $msg) $(,$args)*)};
    ($method:ident notification $name:literal: $msg:literal $(,$args:expr)*) => {#[cfg(feature = "logging")] ::log::$method!(concat!("NOTIFICATION: \"", $name, "\"", $msg) $(,$args)*)};
}
macro_rules! debug {
    (status $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(debug status $name: $msg $(,$args)*)};
    (notification $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(debug notification $name: $msg $(,$args)*)};
}
macro_rules! trace {
    (status $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(trace status $name: $msg $(,$args)*)};
    (notification $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(trace notification $name: $msg $(,$args)*)};
}
macro_rules! info {
    (status $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(info status $name: $msg $(,$args)*)};
    (notification $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(info notification $name: $msg $(,$args)*)};
}
macro_rules! error {
    (status $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(error status $name: $msg $(,$args)*)};
    (notification $name:literal: $msg:literal $(,$args:expr)*) => {$crate::log!(error notification $name: $msg $(,$args)*)};
}
pub(crate) use {debug, trace, info, error, log};