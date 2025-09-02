// mod state;
// mod config;
// mod routes;
// mod notifications;

fn init_state() -> Result<state_management::State, ()> {
    let state = state_management::State::new();
    #[cfg(feature = "dataminer-status-source")]
    state.register_status_provider::<state_management::standard_modules::DataMinerInfoSource>()?;
    #[cfg(feature = "server-status-source")]
    state.register_status_provider::<state_management::standard_modules::WebServerInfoProvider>()?;
    #[cfg(feature = "e-mail-notifications")]
    state.register_notification_provider::<state_management::standard_modules::EmailNotifications>()?;
    #[cfg(feature = "frontend-website")]
    state.register_notification_provider::<state_management::standard_modules::Website>()?;
    Ok(state)
}

#[rocket::launch]
async fn launch() -> _ {
    rocket::build().mount("/", init_state().unwrap())
}

