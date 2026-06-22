#![windows_subsystem = "windows"]

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

#[cfg(windows)]
fn show_error(title: &str, msg: &str) {
    use std::ffi::CString;
    use windows::core::PCSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, MB_OK};

    let title_c = CString::new(title).unwrap_or_default();
    let msg_c = CString::new(msg).unwrap_or_default();
    unsafe {
        MessageBoxA(
            None,
            PCSTR(msg_c.as_ptr().cast()),
            PCSTR(title_c.as_ptr().cast()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn resolve_config_path() -> String {
    if let Some(path) = std::env::args().nth(1) {
        return path;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let cfg = parent.join("config.toml");
            if cfg.exists() {
                return cfg.to_string_lossy().into_owned();
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let cfg = cwd.join("config.toml");
        if cfg.exists() {
            return cfg.to_string_lossy().into_owned();
        }
    }

    "config.toml".to_string()
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tracing::info!("rust-tiling-window-manager v{}", env!("CARGO_PKG_VERSION"));

    let config_path = resolve_config_path();

    tracing::info!("loading config from: {}", config_path);

    let config = config::Config::from_file(&config_path)
        .map_err(|e| {
            let msg = format!("Failed to load config: {}", e);
            #[cfg(windows)]
            show_error("rtwm Error", &msg);
            anyhow::anyhow!("{}", msg)
        })?;

    let app = app::App::new(config);

    if let Err(e) = app.run() {
        tracing::error!("application error: {}", e);
        let msg = format!("{}", e);
        #[cfg(windows)]
        show_error("rtwm Error", &msg);
        return Err(anyhow::anyhow!("{}", msg));
    }

    Ok(())
}
