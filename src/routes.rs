mod api;
pub mod websocket;
mod frontend;

use std::sync::Arc;
use crate::state::State;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![miner_ping, reload_config, frontend::index, frontend::style, frontend::frontend_js, frontend::frontend_wasm, api::all_statuses, websocket::websocket]
}

#[rocket::post("/ping?<id>")]
pub fn miner_ping(id: String, miner_states: &rocket::State<Arc<State>>) -> rocket::response::content::RawJson<String> {
    let state = Arc::clone(miner_states.inner());
    tokio::spawn(async move {
        let went_online = state.miner_ping(&id).await;
        websocket::WebSocket::broadcast(api_types::WebSocketMessage::MinerPing(id.clone())).await;
        if went_online {
            crate::notifications::send_notification(&id, crate::notifications::NotificationReason::WentOnline, crate::notifications::Type::DataMiner, &state);
        }
        if let Some(timeout_duration) = state.get_timeout_period(&id).await {
            tokio::time::sleep(timeout_duration.to_std().unwrap()).await;
            if !state.is_online(&id).await && !state.is_marked_offline(&id).await {
                state.mark_offline(&id).await;
                crate::notifications::send_notification(&id, crate::notifications::NotificationReason::WentOffline, crate::notifications::Type::DataMiner, &state);
            }
        }
    });
    rocket::response::content::RawJson("{}".to_string())
}
#[rocket::get("/reload_config")]
pub async fn reload_config(state: &rocket::State<Arc<State>>) -> rocket::response::Redirect {
    state.load_config().await.unwrap();
    rocket::response::Redirect::to("/")
}
