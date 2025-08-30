use crate::config::LiveConfig;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

#[derive(Default, Debug, Clone)]
pub struct DataMinerStatus {
    last_ping: chrono::DateTime<chrono::Local>,
}
impl DataMinerStatus {
    pub fn new() -> DataMinerStatus {
        Self {
            last_ping: chrono::Local::now(),
        }
    }
    pub fn ping(&mut self) {
        self.last_ping = chrono::Local::now();
    }
    pub fn is_online(&self, timeout: chrono::Duration) -> bool {
        let now = chrono::Local::now();
        (now - self.last_ping) < timeout
    }
    pub fn get_last_ping(&self) -> chrono::DateTime<chrono::Local> {
        self.last_ping
    }
}

#[derive(Default)]
pub struct State {
    miner_stati: RwLock<HashMap<String, RwLock<DataMinerStatus>>>,
    miner_config: RwLock<HashMap<String, LiveConfig>>,
    marked_offline: RwLock<HashSet<String>>,
}
impl State {
    const CONFIG_FILE: &'static str = "config.yaml";
    /// # Returns
    /// Whether the miner just went online, aka, whether we need to send a notification.
    pub async fn miner_ping(&self, id: &String) -> bool {
        let lock = self.miner_stati.read().await;
        let Some(miner) = lock.get(id) else {
            self.miner_stati.write().await.insert(id.clone(), RwLock::new(DataMinerStatus::new()));
            return true;
        };
        let updated = {
            match self.get_timeout_period(id).await {
                None => false,
                Some(duration) => miner.read().await.is_online(duration),
            }
        };
        self.marked_offline.write().await.insert(id.clone());
        miner.write().await.ping();
        updated
    }
    pub async fn get_timeout_period(&self, miner_id: &String) -> Option<chrono::Duration> {
        self.miner_config.read().await.get(miner_id)?.timeout_period
    }
    pub async fn load_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(Self::CONFIG_FILE)?;
        let data: HashMap<String, LiveConfig> = serde_yaml::from_reader(file)?;
        let mut lock = self.miner_config.write().await;
        *lock = data;
        Ok(())
    }
    pub async fn is_online(&self, miner_id: &String) -> bool {
        let lock = self.miner_stati.read().await;
        let Some(miner) = lock.get(miner_id) else { return false; };
        let Some(timeout_period) = self.get_timeout_period(miner_id).await else { return true; };
        miner.read().await.is_online(timeout_period)
    }
    pub async fn mark_offline(&self, miner_id: &String) {
        let mut lock = self.marked_offline.write().await;
        lock.insert(miner_id.clone());
    }
    pub async fn is_marked_offline(&self, miner_id: &String) -> bool {
        let lock = self.marked_offline.read().await;
        lock.contains(miner_id)
    }
}