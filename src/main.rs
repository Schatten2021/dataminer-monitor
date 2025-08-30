use crate::state::State;

mod state;
mod config;
mod routes;

#[rocket::launch]
async fn launch() -> _ {
    rocket::build()
        .manage(std::sync::Arc::new(State::new().await.expect("unable to initialize state")))
        .mount("/", rocket::routes![routes::miner_ping])
}

