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
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;
use tracing::level_filters::LevelFilter;

#[cfg(debug_assertions)]
const LEVEL: LevelFilter = LevelFilter::TRACE;
#[cfg(not(debug_assertions))]
const LEVEL: LevelFilter = LevelFilter::INFO;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LEVEL)
        .init();

    let args = Args::parse();
    info!("parsed args: {args:?}");

    let server = server::Server::new(args.config_file);
    // wait for USR1 signal to reload config
    let mut signal = signal(SignalKind::user_defined1()).expect("unable to register SIGUSR1 signal handler");
    let server_ = server.clone();
    tokio::spawn(async move {
        while signal.recv().await.is_some() {
            server.reload_config();
        }
    });
    let server = server_;

    macro_rules! component {
        (if $feature:literal: $component:ident) => {
            #[cfg(feature = $feature)]
            server.add_component::<::default_components::$component>();
        };
        (if $feature:literal: notify $component:ident) => {
            #[cfg(feature = $feature)]
            server.add_notification_provider::<::default_components::$component>();
        };
    }
    component!(if "api": Api);
    component!(if "websockets": notify Websockets);
    component!(if "frontend": Frontend);

    component!(if "ntfy-notifications": notify NtfyNotificationProvider);
    component!(if "email-notifications": notify EmailNotificationProvider);

    component!(if "dataminer-status": DataminerStatus);
    component!(if "minecraft-status": MinecraftStatus);
    component!(if "website-status": WebsiteStatuse);

    let router = axum::Router::new()
        .route("/", any(server.clone()))
        .route("/{*any}", any(server.clone()));
    let listener = tokio::net::TcpListener::bind((args.host.clone(), args.port)).await.unwrap();
    info!("listening on http://{}:{}", args.host, args.port);
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