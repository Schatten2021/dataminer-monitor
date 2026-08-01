use axum::routing::any;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();

    let server = server::Server::new("config.toml".into());
    server.add_component::<default_components::WebsiteStatuse>()
        .add_notification_provider::<default_components::Api>()
        .add_component::<default_components::Frontend>()
        .add_component::<default_components::DataminerStatus>()
        .add_component::<default_components::MinecraftStatus>();
    let router = axum::Router::new()
        .route("/", any(server.clone()))
        .route("/{*any}", any(server.clone()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, router).await.unwrap();

}