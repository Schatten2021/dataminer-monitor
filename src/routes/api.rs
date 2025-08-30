use api_types::*;
use crate::state::State;

#[rocket::post("/api/all_statuses")]
pub async fn all_statuses(state: &rocket::State<std::sync::Arc<State>>) -> rocket::serde::json::Json<Vec<DataminerStatus>> {
    rocket::serde::json::Json(state.get_all_stati().await)
}