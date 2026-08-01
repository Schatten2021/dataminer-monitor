

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let app_div = document.get_element_by_id("app").expect("should have a app element");
    yew::Renderer::<Main>::with_root(app_div).render();
    web_sys::console::log_1(&"running_app".into());
}
pub use wasm_bindgen_futures::spawn_local as spawn;
use yew::{html, Context, Html};

struct Main;
impl yew::Component for Main {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <h1> {"Welcome"} </h1>
        }
    }
}