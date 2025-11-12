mod element_status_display;
mod category;

use std::collections::HashMap;
use yew::prelude::*;
use api_types::WebSocketMessage;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MinerStatus {
    pub id: String,
    pub name: String,
    pub last_seen: Option<chrono::DateTime<chrono::Local>>,
    pub is_online: bool,
}
impl From<api_types::ElementStatus> for MinerStatus {
    fn from(status: api_types::ElementStatus) -> Self {
        Self {
            id: status.id,
            name: status.name,
            last_seen: status.last_ping.map(|p| p.with_timezone(&chrono::Local)),
            is_online: status.is_online,
        }
    }
}

pub enum Message {
    StatusesReceived(api_types::AllStatiResponse),
    WSMessage(api_types::WebSocketMessage)
}

#[derive(Default)]
pub struct Main {
    statuses: HashMap<String, Vec<MinerStatus>>,
}

impl Main {
    fn get_status_mut(&mut self, type_id: &str, miner_id: &str) -> Option<&mut MinerStatus> {
        self.statuses.get_mut(type_id)?.iter_mut().find(|s| s.id == miner_id)
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
                self.statuses = statuses.into_iter().map(|(k, v)| (k, v.into_iter().map(Into::into).collect())).collect();
            },
            Message::WSMessage(message) => {
                match message {
                    WebSocketMessage::MinerStatusChange(api_types::StatusUpdate { type_id, id, new_status }) => {
                        let Some(status) = self.get_status_mut(&type_id, &id) else { return false };
                        status.is_online = new_status;
                    },
                    WebSocketMessage::MinerPing { type_id, id: miner_id } => {
                        let Some(status) = self.get_status_mut(&type_id, &miner_id) else { return false };
                        status.last_seen = Some(chrono::Local::now());
                    }
                }
            }
        }
        true
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.statuses.iter().map(|(id, values)| {
            html!{
                <category::CategoryDisplay category_id={id.clone()} stati={values.clone()} />
            }
        }).collect::<Html>()
    }
}