use yew::prelude::*;

#[derive(yew::Properties, PartialEq, Clone)]
pub struct Properties {
    pub category_id: String,
    pub stati: Vec<crate::interface::MinerStatus>
}
pub struct CategoryDisplay;
impl Component for CategoryDisplay {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let displays = props.stati.iter().map(|status| {
            html!{ <super::element_status_display::ElementStatusDisplay miner_status={status.clone()} /> }
        }).collect::<Html>();
        html!{
            <div class="status-category">
                <h1 class="status-category-id">{&props.category_id}</h1>
                {displays}
            </div>
        }
    }
}