use api_types::WebSocketMessage;
use gloo_net::websocket::Message;

pub fn build_url(url: &str) -> String {
    let base: String = web_sys::window().unwrap().location().href().unwrap();
    web_sys::Url::new_with_base(url, &base).unwrap().href()
}
pub async fn post_request<Request: serde::Serialize, Response: for<'a> serde::Deserialize<'a>>(path: &'static str, data: &Request) -> Result<Response, gloo_net::Error> {
    gloo_net::http::Request::post(&*build_url(path))
        .json(data)?
        .send().await?
        .json().await
}
pub async fn get_all_stati() -> Result<Vec<api_types::DataminerStatus>, gloo_net::Error> {
    post_request("api/all_statuses", &()).await
}
pub fn subscribe(callback: yew::Callback<WebSocketMessage>) -> Result<(), wasm_bindgen::JsError> {
    println!("subscribing to websocket");
    let mut socket = gloo_net::websocket::futures::WebSocket::open(&*build_url("ws"))?;
    crate::spawn(async move {
        use futures_util::StreamExt;
        while let Some(Ok(message)) = socket.next().await {
            match message {
                Message::Text(text) => {
                    let message = serde_json::from_str::<WebSocketMessage>(&text).expect("unable to parse message from server");
                    callback.emit(message)
                }
                Message::Bytes(_) => {}
            }
        }
    });
    Ok(())
}