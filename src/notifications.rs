use std::sync::Arc;
use crate::state::State;

#[derive(Clone, Copy, Debug)]
pub enum NotificationReason {
    WentOnline,
    WentOffline,
}
#[derive(Clone, Copy, Debug)]
pub enum Type {
    DataMiner,
}
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::DataMiner => "Dataminer",
        })
    }
}
pub fn send_notification(item_id: &str, reason: NotificationReason, source: Type, _state: &Arc<State>) {
    #[cfg(feature = "e-mail-notifications")]
    {
        let state = _state.clone();
        let id = item_id.to_string();
        tokio::spawn(async move {
            let subject = match reason {
                NotificationReason::WentOnline => format!("{source} `{id}` is back online"),
                NotificationReason::WentOffline => format!("{source} `{id}` went offline!"),
            };
            let body = match reason {
                NotificationReason::WentOnline => format!("<h1>The {source} <code>{id}</code> is now back online. Live is good.</h1>"),
                NotificationReason::WentOffline => format!("<h1>The {source} <code>{id}</code> just went offline! They need a checkin!</h1>"),
            };
            if let Err(e) = send_email(state, subject, body).await {
                rocket::log::private::error!("Failed to send E-Mail: {e}");
            };
        });
    }
    let id = item_id.to_string();
    tokio::spawn(async move {
        let info = api_types::MinerStatusChange { id, is_online: match reason {
            NotificationReason::WentOnline => true,
            NotificationReason::WentOffline => false,
        } };
        crate::routes::websocket::WebSocket::broadcast(api_types::WebSocketMessage::MinerStatusChange(info)).await;
    });

}
#[cfg(feature = "e-mail-notifications")]
async fn send_email(state: Arc<State>, subject: String, body: impl lettre::message::IntoBody + Clone) -> Result<(), Box<dyn std::error::Error>> {
    use lettre::Transport;
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