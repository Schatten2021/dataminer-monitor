// mod state;
// mod config;
// mod routes;
// mod notifications;

fn init_state() -> Result<state_management::State, ()> {
    state_management::State::new()
        .register_status_provider::<state_management::standard_modules::DataMinerInfoSource>()?
        .register_status_provider::<state_management::standard_modules::WebServerInfoProvider>()?
        .register_notification_provider::<state_management::standard_modules::EmailNotifications>()?
        .register_notification_provider::<state_management::standard_modules::Website>()
}

#[rocket::launch]
async fn launch() -> _ {
    rocket::build().mount("/", init_state().unwrap())
}

