#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-traits", derive(serde::Serialize, serde::Deserialize))]
pub struct Notification {
    pub item_name: String,
    pub item_id: String,
    pub reason: NotificationReason,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-traits", derive(serde::Serialize, serde::Deserialize))]
pub enum NotificationReason {
    #[serde(alias="went-online")]
    WentOnline,
    #[serde(alias="went-offline")]
    WentOffline,
    #[serde(alias="ping", alias="pings")]
    Seen,
    Other(String),
}
impl std::fmt::Display for NotificationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::WentOnline => "went online",
            Self::WentOffline => "went offline",
            Self::Seen => "was seen",
            Self::Other(v) => &v, 
        }.fmt(f)
    }
}