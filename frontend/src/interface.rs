use yew::prelude::*;
pub struct Main {

}

impl Component for Main {
    type Message = ();
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {<h1>{"Hello World!"}</h1>}
    }
}