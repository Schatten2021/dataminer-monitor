#[rocket::get("/")]
pub fn index() -> rocket::response::content::RawHtml<String> {
    rocket::response::content::RawHtml(include_str!("../../static/index.html").to_string())
}
#[rocket::get("/static/style.css")]
pub fn style() -> rocket::response::content::RawCss<String> {
    #[allow(unused_mut)]
    let mut result = include_str!("../../static/style.css").to_string();

    #[cfg(feature = "frontend-file-hot-reload")]
    #[cfg(all(feature = "default", debug_assertions))]
    if let Ok(content) = std::fs::read_to_string("static/style.css") {
        result = content
    }

    rocket::response::content::RawCss(result)
}
#[rocket::get("/static/wasm/frontend.js")]
pub fn frontend_js() -> rocket::response::content::RawJavaScript<String> {
    #[allow(unused_mut)]
    let mut result = include_str!("../../static/wasm/frontend.js").to_string();

    #[cfg(feature = "frontend-file-hot-reload")]
    #[cfg(all(feature = "default", debug_assertions))]
    if let Ok(content) = std::fs::read_to_string("static/wasm/frontend.js") {
        result = content;
    }

    rocket::response::content::RawJavaScript(result)
}

#[rocket::get("/static/wasm/frontend_bg.wasm")]
pub async fn frontend_wasm<'a, 'b: 'a>() -> impl rocket::response::Responder<'a, 'b> {
    #[derive(rocket::Responder)]
    #[response(content_type = "application/wasm", status = 200)]
    struct Responder(Vec<u8>);

    #[allow(unused_mut)]
    let mut body = include_bytes!("../../static/wasm/frontend_bg.wasm").to_vec();
    #[cfg(any(feature = "frontend-file-hot-reload", all(feature = "default", debug_assertions)))]
    if let Ok(content) = std::fs::read("static/wasm/frontend_bg.wasm") {
        body = content;
    }

    Responder(body)
}