
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    #[cfg(feature = "e-mail-notifications")]
    pub email: EmailConfig,
    #[serde(default)]
    pub notify: Vec<String>,
    #[serde(default)]
    pub miner: std::collections::HashMap<String, MinerConfig>,
}


#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct MinerConfig {
    pub timeout: chrono::Duration,
}
#[cfg(feature = "e-mail-notifications")]
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server: String,
}