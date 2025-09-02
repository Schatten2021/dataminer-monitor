#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ElementStatus {
    pub id: String,
    pub name: String,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub is_online: bool,
}
pub type AllStatiResponse = std::collections::HashMap<String, Vec<ElementStatus>>;
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StatusUpdate {
    pub type_id: String,
    pub id: String,
    pub new_status: bool,
}

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
    MinerStatusChange(StatusUpdate),
    MinerPing {
        type_id: String,
        miner_id: String,
    },
}