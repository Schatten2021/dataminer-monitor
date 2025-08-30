
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub email: EmailConfig,
    #[serde(default)]
    pub notify: Vec<String>,
    #[serde(default)]
    pub timeouts: std::collections::HashMap<String, TimeoutConfig>,
}


#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct TimeoutConfig {
    pub period: chrono::Duration,
}
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server: String,
}