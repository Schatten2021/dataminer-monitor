use crate::config::{Config, TimeoutConfig};
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
}

pub struct State {
    miner_stati: RwLock<HashMap<String, RwLock<DataMinerStatus>>>,
    miner_config: RwLock<HashMap<String, TimeoutConfig>>,
    marked_offline: RwLock<HashSet<String>>,
    #[cfg(feature = "e-mail-notifications")]
    pub email_config: RwLock<crate::config::EmailConfig>,
    pub notification_targets: RwLock<Vec<String>>,
}
impl State {
    const CONFIG_FILE: &'static str = "config.toml";
    /// # Returns
    /// Whether the miner just went online, aka, whether we need to send a notification.
    pub async fn miner_ping(&self, id: &String) -> bool {
        let lock = self.miner_stati.read().await;
        let Some(miner) = lock.get(id) else {
            drop(lock);
            self.miner_stati.write().await.insert(id.clone(), RwLock::new(DataMinerStatus::new()));
            return true;
        };
        let updated = {
            match self.get_timeout_period(id).await {
                None => false,
                Some(duration) => !miner.read().await.is_online(duration),
            }
        };
        self.marked_offline.write().await.remove(id);
        miner.write().await.ping();
        updated
    }
    pub async fn get_timeout_period(&self, miner_id: &String) -> Option<chrono::Duration> {
        Some(self.miner_config.read().await.get(miner_id)?.period)
    }
    pub async fn load_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data: Config = toml::from_str(&*std::fs::read_to_string(Self::CONFIG_FILE)?)?;
        let mut timout_lock = self.miner_config.write().await;
        *timout_lock = data.timeouts;
        #[cfg(feature = "e-mail-notifications")]
        {
            let mut email_lock = self.email_config.write().await;
            *email_lock = data.email;
        }
        let mut notification_targets_lock = self.notification_targets.write().await;
        *notification_targets_lock = data.notify;
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
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let new_self = Self {
            miner_stati: Default::default(),
            miner_config: Default::default(),
            marked_offline: Default::default(),
            #[cfg(feature = "e-mail-notifications")]
            email_config: Default::default(),
            notification_targets: Default::default(),
        };
        new_self.load_config().await?;
        Ok(new_self)
    }
}
impl State {
    pub async fn get_all_stati(&self) -> Vec<api_types::DataminerStatus> {
        let mut result = Vec::with_capacity(self.miner_stati.read().await.len());
        let mut found_ids = HashSet::new();
        for (id, status) in self.miner_stati.read().await.iter() {
            result.push(api_types::DataminerStatus {
                id: id.clone(),
                last_ping: Some(status.read().await.last_ping.to_utc()),
                timeout_period: self.get_timeout_period(id).await,
            });
            found_ids.insert(id.clone());
        }
        for (id, timeout) in self.miner_config.read().await.iter()
            .filter(|(id, _)| !found_ids.contains(*id)) {
            result.push(api_types::DataminerStatus {
                id: id.clone(),
                last_ping: None,
                timeout_period: Some(timeout.period),
            });
        }
        result.into_iter().collect()
    }
}