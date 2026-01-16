mod config;
mod discord;
mod error;
mod gateway;
mod handler;
mod health;
mod model;
mod repository;
mod service;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::sync::watch;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::gateway::event_loop::{run_gateway, BotConfig, GatewayState};
use crate::health::server::run_health_server;
use crate::service::init_global_notifier;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load .env file if present
    if let Err(e) = dotenvy::dotenv() {
        tracing::debug!("No .env file loaded: {}", e);
    }

    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Configuration loaded");

    // Initialize error notifier (webhook)
    init_global_notifier(config.error_webhook_url.clone());
    if config.error_webhook_url.is_some() {
        tracing::info!("Error webhook notifier initialized");
    }

    // Connect to database
    let pool = match PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Connected to database");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(2);
        }
    };

    // Run migrations
    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => tracing::info!("Database migrations completed"),
        Err(e) => {
            tracing::error!("Failed to run migrations: {}", e);
            std::process::exit(2);
        }
    }

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create gateway state channel
    let (gateway_state_tx, gateway_state_rx) = watch::channel(GatewayState { connected: false });

    // Spawn health server
    let health_pool = pool.clone();
    let health_shutdown_rx = shutdown_rx.clone();
    let health_handle = tokio::spawn(async move {
        if let Err(e) = run_health_server(
            config.health_port,
            health_pool,
            gateway_state_rx,
            health_shutdown_rx,
        )
        .await
        {
            tracing::error!("Health server error: {}", e);
        }
    });

    // Spawn gateway
    let gateway_pool = pool.clone();
    let gateway_shutdown_rx = shutdown_rx.clone();
    let gateway_token = config.discord_token.clone();
    let bot_config = BotConfig {
        name: config.bot_name.clone(),
        description: config.bot_description.clone(),
        developer_id: config.bot_developer_id.clone(),
        github_url: config.bot_github_url.clone(),
    };
    let gateway_handle = tokio::spawn(async move {
        if let Err(e) = run_gateway(
            gateway_token,
            gateway_pool,
            gateway_shutdown_rx,
            gateway_state_tx,
            bot_config,
        )
        .await
        {
            tracing::error!("Gateway error: {}", e);
        }
    });

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C, initiating shutdown");
        }
        _ = async {
            #[cfg(unix)]
            {
                let mut sigterm = tokio::signal::unix::signal(
                    tokio::signal::unix::SignalKind::terminate()
                ).expect("Failed to register SIGTERM handler");
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                std::future::pending::<()>().await;
            }
        } => {
            tracing::info!("Received SIGTERM, initiating shutdown");
        }
    }

    // Signal shutdown
    let _ = shutdown_tx.send(true);

    // Wait for tasks to complete with timeout
    let shutdown_timeout = Duration::from_secs(30);
    tokio::select! {
        _ = async {
            let _ = gateway_handle.await;
            let _ = health_handle.await;
        } => {
            tracing::info!("Graceful shutdown completed");
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            tracing::warn!("Shutdown timeout reached, forcing exit");
        }
    }

    // Close database pool
    pool.close().await;

    tracing::info!("Shutdown complete");
}
