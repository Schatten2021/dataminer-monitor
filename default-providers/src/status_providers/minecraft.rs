use state_management::{NotificationReason, StateHandle};
use std::collections::HashMap;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub java: HashMap<String, JavaConfig>,
}
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JavaConfig {
    pub url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default="java_default_port")]
    pub port: u16,
    #[serde(default="default_timeout")]
    pub interval: chrono::Duration,
}
const fn java_default_port() -> u16 {
    25565
}
const fn default_timeout() -> chrono::Duration {
    chrono::Duration::hours(1)
}
pub struct MinecraftStatusProvider {
    config: Config,
    state: StateHandle,
    stati: Stati,
}
struct Stati {
    java: HashMap<String, (std::sync::Arc<parking_lot::RwLock<Status>>, tokio::task::JoinHandle<()>)>,
}
struct Status {
    last_ping: Option<chrono::DateTime<chrono::Local>>,
    marked_offline: bool,
}
impl MinecraftStatusProvider {
    fn start_pings(state: StateHandle, config: Config) -> Stati {
        Stati {
            java: config.java.into_iter().map(|(id, conf)| {
                let status = std::sync::Arc::new(parking_lot::RwLock::new(Status { last_ping: None, marked_offline: true }));
                let mut ticker = tokio::time::interval(conf.interval.to_std().expect("unable to convert to std time"));

                let cloned_status = status.clone();
                let state = state.clone();
                let cloned_id = id.clone();

                let handle = tokio::spawn(async move {
                    loop {
                        let ping: Result<(), ()> = (|| {
                            let mut conn = std::net::TcpStream::connect((conf.url.clone(), conf.port)).map_err(drop)?;
                            minecraft_net::send_packet(
                                minecraft_net::packets::handshake::upstream::Handshake::new(conf.url.clone(), conf.port, 1),
                                &mut conn,
                                None
                            ).map_err(drop)?;
                            minecraft_net::send_packet(
                                minecraft_net::packets::status::upstream::StatusRequest::new(),
                                &mut conn,
                                None,
                            ).map_err(drop)?;
                            let _response = minecraft_net::receive_packet::<minecraft_net::packets::status::downstream::StatusResponse>(&mut conn, false).map_err(drop)?;
                            Ok(())
                        })();
                        macro_rules! send_notificaion {
                            ($reason:ident) => {
                                state.send_notification(state_management::Notification {
                                    item_name: if conf.name != "" { conf.name.clone()} else {cloned_id.clone()},
                                    item_id: format!("java.{}", cloned_id),
                                    reason: NotificationReason::$reason
                                })
                            }
                        }
                        match ping {
                            Ok(()) => {
                                let mut lock = cloned_status.write();
                                lock.last_ping = Some(chrono::Local::now());
                                if lock.marked_offline {
                                    lock.marked_offline = false;
                                    send_notificaion!(WentOnline);
                                }
                                drop(lock);
                                send_notificaion!(Seen);
                            }
                            Err(_) => {
                                if !cloned_status.read().marked_offline {
                                    cloned_status.write().marked_offline = true;
                                    send_notificaion!(WentOffline);
                                }
                            }
                        }
                        ticker.tick().await;
                    }
                });
                (id, (status, handle))
            }).collect(),
        }
    }
}
impl state_management::StatusProvider for MinecraftStatusProvider {
    const ID: &'static str = "minecraft";
    const NAME: &'static str = "Minecraft Server";
    type Config = Config;

    fn new(state: StateHandle, config: Self::Config) -> Self {
        Self{
            stati: Self::start_pings(state.clone(), config.clone()),
            state,
            config,
        }
    }

    fn update_config(&mut self, config: Self::Config) {
        self.config = config;
        self.stati.java.drain().for_each(|(_, (_, handle))| handle.abort());
        self.stati = Self::start_pings(self.state.clone(), self.config.clone())
    }

    fn current_stati(&self) -> HashMap<String, state_management::Status> {
        self.stati.java.iter()
            .map(|(id, (status, _))| {
                let name = self.config.java.get(id).map(|c| c.name.clone()).unwrap_or_default();
                let name = if name == "" {id.clone()} else {name};
                (format!("java.{id}"), state_management::Status {
                    name,
                    is_online: !status.read().marked_offline,
                    last_seen: status.read().last_ping.as_ref().map(chrono::DateTime::to_utc),
                })
            })
            .collect()
    }
}
