use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub configs: HashMap<String, toml::Value>,

    #[serde(alias="ignore")]
    #[serde(default)]
    /// ignores all components with the given id.
    pub ignored: HashSet<String>,
}
