use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
/// Configuration for the server.
pub struct Config {
    #[serde(flatten)]
    /// Map of all the configs for the different [`crate::Component`]s.
    pub configs: HashMap<String, toml::Value>,

    #[serde(alias="ignore")]
    #[serde(default)]
    /// ignores all components with the given id.
    pub ignored: HashSet<String>,
}
