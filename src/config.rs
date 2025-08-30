
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct LiveConfig {
    pub timeout_period: Option<chrono::Duration>,
}