use std::pin::Pin;
// use std::fmt::{Display, Formatter};
use crate::ComponentHandle;

pub type RequestHandle = Pin<Box<dyn Future<Output=axum::response::Response> + Send >>;

#[async_trait::async_trait]
pub trait Component: Sized + Send + Sync + 'static {
    const ID: &'static str;
    // const TYPE: ComponentType;
    type Config: serde::Serialize + for<'de> serde::Deserialize<'de> + Default;
    type ConfigError: core::error::Error;
    fn init(server: ComponentHandle, config: Self::Config) -> Result<Self, Self::ConfigError>;
    fn reconfigure(&mut self, config: Self::Config) -> Result<(), Self::ConfigError>;
    fn try_handle(&self, request: axum::extract::Request) -> Result<RequestHandle, axum::extract::Request> {
        Err(request)
    }
}
// 
// #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
// pub enum ComponentType {
//     #[serde(alias="statuses", alias="stati")]
//     Status,
//     #[serde(alias="notif", alias="notifs", alias="notifications")]
//     Notification,
// }
// impl Display for ComponentType {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.write_str(match self {
//             ComponentType::Status => "status",
//             ComponentType::Notification => "notification",
//         })
//     }
// }