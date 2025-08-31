mod miner_status_display;

use yew::prelude::*;
use api_types::WebSocketMessage;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MinerStatus {
    pub id: String,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub is_online: bool,
}
impl From<api_types::DataminerStatus> for MinerStatus {
    fn from(status: api_types::DataminerStatus) -> Self {
        Self {
            id: status.id,
            last_seen: status.last_ping,
            is_online: status.last_ping.map_or(false, |p| status.timeout_period.map_or(true, |v| (chrono::Utc::now() - p) < v)),
        }
    }
}

pub enum Message {
    StatusesReceived(Vec<api_types::DataminerStatus>),
    WSMessage(api_types::WebSocketMessage)
}

#[derive(Default)]
pub struct Main {
    statuses: Vec<MinerStatus>,
}

impl Main {
    fn get_status_mut(&mut self, id: &str) -> Option<&mut MinerStatus> {
        self.statuses.iter_mut().find(|v| v.id == id)
    }
}
impl Component for Main {
    type Message = Message;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Message::StatusesReceived);
        crate::spawn(async move {
            callback.emit(crate::api::get_all_stati().await.expect("unable to request stati"))
        });
        crate::api::subscribe(ctx.link().callback(Message::WSMessage)).expect("unable to subscribe to websocket");
        Self::default()
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::StatusesReceived(statuses) => {
                self.statuses = statuses.into_iter().map(Into::into).collect();
            },
            Message::WSMessage(message) => {
                match message {
                    WebSocketMessage::MinerStatusChange(api_types::MinerStatusChange { id, is_online }) => {
                        let Some(status) = self.get_status_mut(&id) else { return false };
                        status.is_online = is_online;
                    },
                    WebSocketMessage::MinerPing(miner_id) => {
                        let Some(status) = self.get_status_mut(&miner_id) else { return false };
                        status.last_seen = Some(chrono::Utc::now());
                    }
                }
            }
        }
        true
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.statuses.iter().map(|status| html!{<miner_status_display::MinerStatusDisplay miner_status={status.clone()} />}).collect()
    }
}