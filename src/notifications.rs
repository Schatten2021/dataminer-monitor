use std::sync::Arc;
use lettre::Transport;
use crate::state::State;

#[derive(Clone, Copy, Debug)]
pub enum NotificationReason {
    WentOnline,
    WentOffline,
}
pub fn send_notification(miner_id: &str, reason: NotificationReason, _state: &Arc<State>) {
    let state = _state.clone();
    let id = miner_id.to_string();
    tokio::spawn(async move {
        let subject = match reason {
            NotificationReason::WentOnline => format!("Dataminer {id} is back online"),
            NotificationReason::WentOffline => format!("Dataminer {id} went offline!"),
        };
        let body = match reason {
            NotificationReason::WentOnline => format!("<h1>The Dataminer {id} is now back online. Live is good.</h1>"),
            NotificationReason::WentOffline => format!("<h1>The Dataminer {id} just went offline! They need a checkin!</h1>"),
        };
        if let Err(e) = send_email(state, subject, body).await {
            rocket::log::private::error!("Failed to send E-Mail: {e}");
        };
    });

    let id = miner_id.to_string();
    tokio::spawn(async move {
        let info = api_types::MinerStatusChange { id, is_online: match reason {
            NotificationReason::WentOnline => true,
            NotificationReason::WentOffline => false,
        } };
        crate::routes::websocket::WebSocket::broadcast(api_types::WebSocketMessage::MinerStatusChange(info)).await;
    });

}
async fn send_email(state: Arc<State>, subject: String, body: impl lettre::message::IntoBody + Clone) -> Result<(), Box<dyn std::error::Error>> {
    let config = state.email_config.read().await.clone();
    let credentials = lettre::transport::smtp::authentication::Credentials::new(config.address.clone(), config.password);

    let mailer = lettre::transport::smtp::SmtpTransport::starttls_relay(&*config.server).expect("unable to start SMTP transport")
        .credentials(credentials)
        .build();
    let lock = state.notification_targets.read().await;
    for target in lock.iter() {
        let email = lettre::Message::builder()
            .from(format!("No Reply <{}>", config.address).parse()?)
            .to(target.parse().expect("failed to parse target E-Mail address"))
            .subject(&*subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body.clone())?;
        mailer.send(&email)?;
    }
    Ok(())
}