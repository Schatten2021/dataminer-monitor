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
pub enum WebSocketMessage {
    MinerStatusChange(StatusUpdate),
    MinerPing {
        type_id: String,
        id: String,
    },
}