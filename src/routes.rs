use std::sync::Arc;
use lettre::Transport;
use crate::state::State;

#[rocket::post("/ping?<id>")]
pub fn miner_ping(id: String, miner_states: &rocket::State<Arc<State>>) -> rocket::response::content::RawJson<String> {
    let state = Arc::clone(miner_states.inner());
    tokio::spawn(async move {
        let went_online = state.miner_ping(&id).await;
        if let Some(timeout_duration) = state.get_timeout_period(&id).await {
            let state = Arc::clone(&state);
            let id = id.clone();
            tokio::spawn(async move {
                tokio::time::sleep(timeout_duration.to_std().expect("failed to convert timeout duration")).await;
                if state.is_online(&id).await { return; } // miner stayed online => nothing to do
                if state.is_marked_offline(&id).await { return; }
                state.mark_offline(&id).await;
                send_email(&state, format!("miner {id} went offline!"), format!(r#"
<h1>The server <code>{id}</code> went offline!</h1>
"#)).await.expect("failed to send email");
            });
        }
        if went_online {
            send_email(&state, format!("miner {id} went online!"), format!(r#"
<h1>The server <code>{id}</code> is back online! Hooray!</h1>"#)).await.expect("failed to send email");
    }
    });
    rocket::response::content::RawJson("{}".to_string())
}

async fn send_email(state: &Arc<State>, subject: String, body: impl lettre::message::IntoBody + Clone) -> Result<(), Box<dyn std::error::Error>> {
    let lock = state.notification_targets.read().await;
    let credentials = {
        let lock = state.email_config.read().await;
        lettre::transport::smtp::authentication::Credentials::new(lock.address.clone(), lock.password.clone())
    };
    let mailer = lettre::transport::smtp::SmtpTransport::starttls_relay(&*state.email_config.read().await.server.clone())?
        .credentials(credentials)
        .build();
    for target in lock.iter() {
        let email = lettre::Message::builder()
            .from("No Reply <no_reply@mail.fms.nrw>".parse()?)
            .to(target.parse()?)
            .subject(&*subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body.clone())?;
        mailer.send(&email)?;
    }
    Ok(())
}