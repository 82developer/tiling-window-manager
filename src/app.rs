use crate::commands::CommandExecutor;
use crate::config::Config;
use crate::error::AppResult;
use crate::hotkeys::HotkeyRegistry;
use crate::infrastructure::win32::hotkey_api::Win32HotkeyApi;
use crate::infrastructure::win32::monitor_api::Win32MonitorApi;
use crate::infrastructure::win32::window_api::Win32WindowApi;
use crate::rules::WindowRule;
use crate::services::layout_service::LayoutService;
use crate::services::monitor_service::MonitorService;
use crate::services::window_service::WindowService;
use crate::services::workspace_service::WorkspaceService;

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&self) -> AppResult<()> {
        tracing::info!("rust-tiling-window-manager v{} starting...", env!("CARGO_PKG_VERSION"));
        tracing::info!(
            "config loaded: gap={}, margin={}, rules={}",
            self.config.layout.gap,
            self.config.layout.margin,
            self.config.rules.len()
        );

        let mut hotkey_registry = HotkeyRegistry::new();
        hotkey_registry.register_from_config(&self.config.hotkeys)?;

        if hotkey_registry.entries().is_empty() {
            tracing::warn!("no hotkeys configured; add hotkeys to config.toml");
        }

        let window_api = Win32WindowApi::new()?;
        let monitor_api = Win32MonitorApi::new()?;
        let monitor_api2 = Win32MonitorApi::new()?;

        let window_service = WindowService::new(window_api, self.config.ignore.clone());
        let monitor_service = MonitorService::new(monitor_api);
        let monitor_service2 = MonitorService::new(monitor_api2);

        let layout_service = LayoutService::new(
            self.config.layout.gap,
            self.config.layout.margin,
            window_service,
            monitor_service,
        );

        let window_api2 = Win32WindowApi::new()?;
        let window_service2 = WindowService::new(window_api2, self.config.ignore.clone());

        let mut workspace_service = WorkspaceService::new(window_service2, 5);

        for rule_entry in &self.config.rules {
            let rule = WindowRule {
                class_pattern: rule_entry.class.clone(),
                title_pattern: rule_entry.title.clone(),
                target_workspace: if rule_entry.workspace > 0 {
                    Some(rule_entry.workspace - 1)
                } else {
                    None
                },
                floating: rule_entry.floating,
            };
            tracing::info!(
                "adding rule: class={:?} title={:?} workspace={:?}",
                rule.class_pattern, rule.title_pattern, rule.target_workspace
            );
            workspace_service.add_rule(rule);
        }

        let executor = CommandExecutor::new(
            layout_service,
            workspace_service,
            monitor_service2,
            self.config.terminal.command.clone(),
        );

        let mut hotkey_api = Win32HotkeyApi::new()?;
        let mut registered_count = 0u32;
        let mut failed_count = 0u32;
        let mut registry = HotkeyRegistry::new();

        for entry in hotkey_registry.into_entries() {
            tracing::info!(
                "registering hotkey: id={} action={:?} desc={}",
                entry.id, entry.action, entry.description,
            );

            match hotkey_api.register_hotkey(entry.id, entry.modifiers, entry.vk) {
                Ok(()) => {
                    registry.add_entry(entry);
                    registered_count += 1;
                }
                Err(e) => {
                    failed_count += 1;
                    tracing::warn!(
                        "failed to register hotkey '{}': {}. It may be in use by another application.",
                        entry.description, e
                    );
                }
            }
        }

        tracing::info!(
            "hotkey registration: {} ok, {} failed, {} total",
            registered_count, failed_count, registered_count + failed_count
        );

        if registered_count == 0 {
            return Err(crate::error::AppError::Hotkey(
                "no hotkeys registered. Edit config.toml to use different key combinations.".to_string()
            ));
        }

        if failed_count > 0 {
            tracing::warn!(
                "{} hotkey(s) failed. App continues with {} active hotkeys.",
                failed_count, registered_count
            );
        }

        tracing::info!(
            "{} hotkeys active. Entering message loop...",
            registry.entries().len()
        );

        let result = hotkey_api.run_message_loop(|hotkey_id| {
            let action = match registry.find_by_id(hotkey_id) {
                Some(a) => a,
                None => {
                    tracing::warn!("unknown hotkey id: {}", hotkey_id);
                    return true;
                }
            };

            if let Err(e) = executor.execute(action) {
                if executor.should_quit(action) {
                    tracing::info!("quit requested via hotkey");
                    hotkey_api.request_quit();
                    return false;
                }
                tracing::error!("error executing {:?}: {}", action, e);
            }

            true
        });

        tracing::info!("rust-tiling-window-manager shutting down");
        result
    }
}
