use crate::interface::MinerStatus;
use yew::{html, Component, Context, Html};

#[derive(yew::Properties, Clone, PartialEq)]
pub struct Properties {
    pub miner_status: MinerStatus,
}

pub struct MinerStatusDisplay;
impl Component for MinerStatusDisplay {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let status = &ctx.props().miner_status;
        let last_seen = match status.last_seen {
            None => "Hasn't pinged yet...".to_string(),
            Some(t) => t.format("%d.%m.%Y %H:%M:%S").to_string(),
        };
        html!(
            <div class="miner">
                <div class="miner-name">
                    <span class={format!("miner-status {}", status.is_online.then_some("miner-status-active").unwrap_or_default())}>{"⬤"}</span>
                    {&status.name}
                </div>
                <div class="miner-last-seen">{"Last seen: "}{last_seen}</div>
            </div>
        )
    }
}