pub mod app;
pub mod config;
pub mod db;
pub mod dto;
pub mod entity;
pub mod error;
pub mod openapi;
pub mod repository;
pub mod routes;

use std::net::{IpAddr, SocketAddr};

use anyhow::Context;
use config::Settings;
use db::init_database;
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let settings = Settings::load().context("failed to load settings")?;
    let db = init_database(&settings.database_url)
        .await
        .context("failed to initialize database")?;

    let app = routes::router(db);
    let addr = SocketAddr::from((settings.host.parse::<IpAddr>()?, settings.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("plan api listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "plan_api=info,tower_http=info".into());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
