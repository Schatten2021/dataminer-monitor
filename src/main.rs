// mod state;
// mod config;
// mod routes;
// mod notifications;

fn init_state() -> Result<state_management::State, ()> {
    let state = state_management::State::default();
    #[cfg(feature = "dataminer-status-source")]
    state.register_status_provider::<default_providers::status_providers::DataminerStatusProvider>()?;
    #[cfg(feature = "server-status-source")]
    state.register_status_provider::<default_providers::status_providers::ServerStatusProvider>()?;
    #[cfg(feature = "minecraft-server-status-source")]
    state.register_status_provider::<default_providers::status_providers::MinecraftStatusProvider>()?;
    #[cfg(feature = "e-mail-notifications")]
    state.register_notification_provider::<default_providers::notification_providers::EmailNotificationProvider>()?;
    #[cfg(feature = "frontend-website")]
    state.register_notification_provider::<default_providers::notification_providers::WebsiteNotificationProvider>()?;
    Ok(state)
}

#[rocket::launch]
async fn launch() -> _ {
    rocket::build().mount("/", init_state().unwrap())
}

