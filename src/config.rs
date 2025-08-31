
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Config {
    #[cfg(feature = "e-mail-notifications")]
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
#[cfg(feature = "e-mail-notifications")]
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server: String,
}