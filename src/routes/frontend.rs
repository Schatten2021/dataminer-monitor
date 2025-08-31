#[rocket::get("/")]
pub fn index() -> rocket::response::content::RawHtml<String> {
    rocket::response::content::RawHtml(include_str!("../../static/index.html").to_string())
}
#[rocket::get("/static/style.css")]
pub fn style() -> rocket::response::content::RawCss<String> {
    rocket::response::content::RawCss(
        std::fs::read_to_string("static/style.css").unwrap_or(include_str!("../../static/style.css").to_string())
    )
}
#[rocket::get("/static/wasm/frontend.js")]
pub fn frontend_js() -> rocket::response::content::RawJavaScript<String> {
    rocket::response::content::RawJavaScript(
        std::fs::read_to_string("static/wasm/frontend.js").unwrap_or(include_str!("../../static/wasm/frontend.js").to_string())
    )
}

#[rocket::get("/static/wasm/frontend_bg.wasm")]
pub async fn frontend_wasm<'a, 'b: 'a>() -> impl rocket::response::Responder<'a, 'b> {
    #[derive(rocket::Responder)]
    #[response(content_type = "application/wasm", status = 200)]
    struct Responder(Vec<u8>);
    let body = std::fs::read("static/wasm/frontend_bg.wasm").map(|v| v).unwrap_or(include_bytes!("../../static/wasm/frontend_bg.wasm").to_vec());
    Responder(body)
}