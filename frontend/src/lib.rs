mod interface;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let app_div = document.get_element_by_id("app").expect("should have a app element");
    yew::Renderer::<interface::Main>::with_root(app_div).render();
    web_sys::console::log_1(&"running_app".into());
}