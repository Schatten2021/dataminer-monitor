mod api;

use std::sync::Arc;
use lettre::Transport;
use crate::state::State;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![miner_ping, frontend::index, frontend::style, frontend::frontend_js, frontend::frontend_wasm, api::all_statuses]
}

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

mod frontend {
    #[rocket::get("/")]
    pub fn index() -> rocket::response::content::RawHtml<String> {
        rocket::response::content::RawHtml(include_str!("../static/index.html").to_string())
    }
    #[rocket::get("/static/style.css")]
    pub fn style() -> rocket::response::content::RawCss<String> {
        rocket::response::content::RawCss(
            std::fs::read_to_string("static/style.css").unwrap_or(include_str!("../static/style.css").to_string())
        )
    }
    #[rocket::get("/static/wasm/frontend.js")]
    pub fn frontend_js() -> rocket::response::content::RawJavaScript<String> {
        rocket::response::content::RawJavaScript(
            std::fs::read_to_string("static/wasm/frontend.js").unwrap_or(include_str!("../static/wasm/frontend.js").to_string())
        )
    }

    #[rocket::get("/static/wasm/frontend_bg.wasm")]
    pub async fn frontend_wasm<'a, 'b: 'a>() -> impl rocket::response::Responder<'a, 'b> {
        #[derive(rocket::Responder)]
        #[response(content_type = "application/wasm", status = 200)]
        struct Responder(Vec<u8>);
        let body = std::fs::read("static/wasm/frontend_bg.wasm").map(|v| v).unwrap_or(include_bytes!("../static/wasm/frontend_bg.wasm").to_vec());
        Responder(body)
    }
}
