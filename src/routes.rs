use std::sync::Arc;
use crate::state::State;

#[rocket::post("/ping?<id>")]
pub async fn miner_ping(id: String, miner_states: &rocket::State<Arc<State>>) {
    let went_online = miner_states.miner_ping(&id).await;
    if let Some(timeout_duration) = miner_states.get_timeout_period(&id).await {
        let states = Arc::clone(miner_states);
        let id = id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(timeout_duration.to_std().expect("failed to convert timeout duration")).await;
            if states.is_online(&id).await { return; } // miner stayed online => nothing to do
            if states.is_marked_offline(&id).await { return; }
            states.mark_offline(&id).await;
            todo!("send out notification that miner went offline")
        });
    }
    if went_online {
        todo!("send out notification (E-Mail)");
    }
}