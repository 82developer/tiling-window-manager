mod actions;
mod app;
mod commands;
mod config;
mod error;
mod hotkeys;
mod infrastructure;
mod layout;
mod monitor;
mod rules;
mod services;
mod tree;
mod window;
mod workspace;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tracing::info!("rust-tiling-window-manager v{}", env!("CARGO_PKG_VERSION"));

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    tracing::info!("loading config from: {}", config_path);

    let config = config::Config::from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let app = app::App::new(config);

    if let Err(e) = app.run() {
        tracing::error!("application error: {}", e);
        return Err(anyhow::anyhow!("{}", e));
    }

    Ok(())
}
