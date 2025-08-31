#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DataminerStatus {
    pub id: String,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub timeout_period: Option<chrono::Duration>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct MinerStatusChange {
    pub id: String,
    pub is_online: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WebSocketMessage {
    MinerStatusChange(MinerStatusChange),
}