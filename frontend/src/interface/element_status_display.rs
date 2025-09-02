use crate::interface::MinerStatus;
use yew::{html, Component, Context, Html};

#[derive(yew::Properties, Clone, PartialEq)]
pub struct Properties {
    pub miner_status: MinerStatus,
}

pub struct ElementStatusDisplay;
impl Component for ElementStatusDisplay {
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
            <div class="status-display">
                <div class="status-display-name">
                    <span class={format!("status {}", status.is_online.then_some("status-active").unwrap_or_default())}>{"⬤"}</span>
                    {&status.name}
                </div>
                <div class="status-display-seen">{"Last seen: "}{last_seen}</div>
            </div>
        )
    }
}