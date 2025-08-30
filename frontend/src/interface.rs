use yew::prelude::*;

pub enum Message {
    StatusesReceived(Vec<api_types::DataminerStatus>)
}

#[derive(Default)]
pub struct Main {
    statuses: Option<Vec<api_types::DataminerStatus>>,
}

impl Component for Main {
    type Message = Message;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(Message::StatusesReceived);
        crate::spawn(async move {
            callback.emit(crate::api::get_all_stati().await.expect("unable to request stati"))
        });
        Self::default()
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::StatusesReceived(statuses) => {
                self.statuses = Some(statuses);
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.statuses {
            None => html!{ <h1>{"Loading stati"}</h1>},
            Some(stati) => {
                stati.iter()
                    .map(|status| html! {
                        <div class="status">
                            <h2 class="status-name">{&status.id}</h2>
                            <p>
                                {
                                    match status.last_ping.as_ref() {
                                        None => "Never seen before :(".to_string(),
                                        Some(dt) => format!("last seen: {}", dt.with_timezone(&chrono::Local))
                                    }
                                }
                            </p>
                            <p>{
                                match &status.last_ping {
                                    None => "Doesn't timeout :)".to_string(),
                                    Some(dt) => format!("timeouts after: {}", dt.with_timezone(&chrono::Local)),
                                }
                            }</p>
                        </div>
                        })
                    .collect::<Html>()
            },
        }
    }
}