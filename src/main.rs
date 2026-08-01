#![doc=include_str!("../README.md")]


#![cfg_attr(not(debug_assertions), deny(missing_docs))]
#![cfg_attr(debug_assertions, warn(missing_docs))]
#![warn(clippy::pedantic)]
#![warn(clippy::complexity, clippy::suspicious, clippy::perf, clippy::style, clippy::allow_attributes_without_reason)]
#![allow(
clippy::needless_continue,
reason = "adding a `continue` often makes the code easier to read."
)]
#![allow(
clippy::missing_errors_doc,
clippy::doc_markdown,
reason = "don't want these lints."
)]
#![cfg_attr(not(debug_assertions), deny(clippy::undocumented_unsafe_blocks))]
#![cfg_attr(debug_assertions, warn(clippy::undocumented_unsafe_blocks))]


use axum::routing::any;
use clap::Parser;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();

    let args = Args::parse();

    let server = server::Server::new(args.config_file);
    server.add_component::<default_components::WebsiteStatuse>()
        .add_component::<default_components::Frontend>()
        .add_component::<default_components::DataminerStatus>()
        .add_component::<default_components::MinecraftStatus>()
        .add_notification_provider::<default_components::EmailNotificationProvider>()
        .add_notification_provider::<default_components::NtfyNotificationProvider>();
    let router = axum::Router::new()
        .route("/", any(server.clone()))
        .route("/{*any}", any(server.clone()));
    let listener = tokio::net::TcpListener::bind((args.host, args.port)).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[derive(clap::Parser, Debug)]
#[command(version, about="a custom status server")]
struct Args {
    /// The path of the config file
    #[arg(short, long, alias="config", default_value="config.toml")]
    config_file: PathBuf,

    /// The host (*excluding port*) to bind to.
    #[arg(short='b', long="bind", alias="host", default_value="0.0.0.0")]
    host: String,

    /// The port to bind to.
    #[arg(short, long, default_value_t=5000)]
    port: u16,
}