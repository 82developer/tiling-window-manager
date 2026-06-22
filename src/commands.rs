use crate::actions::Action;
use crate::error::AppError;
use crate::layout::{LayoutType, Rect};
use crate::services::layout_service::LayoutService;
use crate::services::monitor_service::MonitorService;
use crate::services::workspace_service::WorkspaceService;
use crate::tree::{FocusDirection, Layout};
use std::process::Command;

pub struct CommandExecutor {
    layout_service: LayoutService,
    workspace_service: std::sync::Mutex<WorkspaceService>,
    monitor_service: MonitorService,
    terminal_command: String,
}

impl CommandExecutor {
    pub fn new(
        layout_service: LayoutService,
        workspace_service: WorkspaceService,
        monitor_service: MonitorService,
        terminal_command: String,
    ) -> Self {
        Self {
            layout_service,
            workspace_service: std::sync::Mutex::new(workspace_service),
            monitor_service,
            terminal_command,
        }
    }

    fn ws(&self) -> Result<std::sync::MutexGuard<'_, WorkspaceService>, AppError> {
        self.workspace_service
            .lock()
            .map_err(|e| AppError::Service(format!("mutex error: {}", e)))
    }

    fn get_work_area(&self) -> Result<Rect, AppError> {
        self.monitor_service
            .get_work_area_for_foreground_window()
            .or_else(|_| Ok(Rect::new(0, 0, 1920, 1040)))
    }

    pub fn execute(&self, action: Action) -> Result<(), AppError> {
        tracing::info!("executing action: {:?}", action);

        {
            if let Ok(mut ws) = self.ws() {
                ws.sync_focus();
            }
        }

        match action {
            Action::MoveLeft => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                ws.move_focused(&FocusDirection::LEFT, area);
                ws.apply_current_layout(area, 8)
            }
            Action::MoveDown => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                ws.move_focused(&FocusDirection::DOWN, area);
                ws.apply_current_layout(area, 8)
            }
            Action::MoveUp => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                ws.move_focused(&FocusDirection::UP, area);
                ws.apply_current_layout(area, 8)
            }
            Action::MoveRight => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                ws.move_focused(&FocusDirection::RIGHT, area);
                ws.apply_current_layout(area, 8)
            }
            Action::Fullscreen => self.layout_service.apply_layout(LayoutType::Fullscreen),
            Action::Center => self.layout_service.apply_layout(LayoutType::Centered),
            Action::LaunchTerminal => self.launch_terminal(),

            Action::LayoutLeftHalf => self.layout_service.apply_layout(LayoutType::LeftHalf),
            Action::LayoutRightHalf => self.layout_service.apply_layout(LayoutType::RightHalf),
            Action::LayoutTopHalf => self.layout_service.apply_layout(LayoutType::TopHalf),
            Action::LayoutBottomHalf => self.layout_service.apply_layout(LayoutType::BottomHalf),

            Action::LayoutMonocle => {
                let mut ws = self.ws()?;
                ws.set_layout(Layout::Monocle);
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::LayoutStacking => {
                let mut ws = self.ws()?;
                ws.set_layout(Layout::Stacking);
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::LayoutSplitH => {
                let mut ws = self.ws()?;
                ws.set_layout(Layout::SplitH);
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::LayoutSplitV => {
                let mut ws = self.ws()?;
                ws.set_layout(Layout::SplitV);
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }

            Action::ToggleSplit => {
                let mut ws = self.ws()?;
                ws.toggle_split();
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::SplitHorizontal => {
                let mut ws = self.ws()?;
                ws.set_split_horizontal();
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::SplitVertical => {
                let mut ws = self.ws()?;
                ws.set_split_vertical();
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }

            Action::FocusLeft => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                if let Some(target) = ws.current_tree_mut().focus_direction(&FocusDirection::LEFT, area) {
                    ws.current_tree_mut().focus_container(target);
                    if let Some(hwnd) = ws.current_tree_mut().focused_hwnd() {
                        ws.focus_window(hwnd)?;
                    }
                }
                Ok(())
            }
            Action::FocusDown => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                if let Some(target) = ws.current_tree_mut().focus_direction(&FocusDirection::DOWN, area) {
                    ws.current_tree_mut().focus_container(target);
                    if let Some(hwnd) = ws.current_tree_mut().focused_hwnd() {
                        ws.focus_window(hwnd)?;
                    }
                }
                Ok(())
            }
            Action::FocusUp => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                if let Some(target) = ws.current_tree_mut().focus_direction(&FocusDirection::UP, area) {
                    ws.current_tree_mut().focus_container(target);
                    if let Some(hwnd) = ws.current_tree_mut().focused_hwnd() {
                        ws.focus_window(hwnd)?;
                    }
                }
                Ok(())
            }
            Action::FocusRight => {
                let mut ws = self.ws()?;
                let area = self.get_work_area()?;
                if let Some(target) = ws.current_tree_mut().focus_direction(&FocusDirection::RIGHT, area) {
                    ws.current_tree_mut().focus_container(target);
                    if let Some(hwnd) = ws.current_tree_mut().focused_hwnd() {
                        ws.focus_window(hwnd)?;
                    }
                }
                Ok(())
            }

            Action::DetectWindow => {
                tracing::info!("--- detect window triggered ---");
                let mut ws = self.ws()?;
                ws.detect_and_manage_window()?;
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)?;
                tracing::info!("--- layout applied ---");
                Ok(())
            }

            Action::NextWorkspace => {
                let mut ws = self.ws()?;
                ws.next_workspace()?;
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::PreviousWorkspace => {
                let mut ws = self.ws()?;
                ws.previous_workspace()?;
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)
            }
            Action::Workspace1 => self.switch_workspace(0),
            Action::Workspace2 => self.switch_workspace(1),
            Action::Workspace3 => self.switch_workspace(2),
            Action::Workspace4 => self.switch_workspace(3),
            Action::Workspace5 => self.switch_workspace(4),

            action @ (Action::MoveToWorkspace1
            | Action::MoveToWorkspace2
            | Action::MoveToWorkspace3
            | Action::MoveToWorkspace4
            | Action::MoveToWorkspace5) => {
                let target = match action {
                    Action::MoveToWorkspace1 => 0,
                    Action::MoveToWorkspace2 => 1,
                    Action::MoveToWorkspace3 => 2,
                    Action::MoveToWorkspace4 => 3,
                    Action::MoveToWorkspace5 => 4,
                    _ => unreachable!(),
                };
                let hwnd = self.layout_service.get_foreground_hwnd();
                if hwnd != 0 {
                    let mut ws = self.ws()?;
                    ws.remove_window_from_current(hwnd);
                    ws.switch_to(target)?;
                    ws.add_window_to_current(hwnd);
                    let area = self.get_work_area()?;
                    ws.apply_current_layout(area, 8)?;
                }
                Ok(())
            }

            Action::Reset => {
                tracing::info!("--- reset triggered ---");
                let mut ws = self.ws()?;
                ws.reset()?;
                let area = self.get_work_area()?;
                ws.apply_current_layout(area, 8)?;
                tracing::info!("--- reset complete ---");
                Ok(())
            }

            Action::Quit => {
                tracing::info!("quit action received");
                Err(AppError::General("quit requested".to_string()))
            }
        }
    }

    pub fn should_quit(&self, action: Action) -> bool {
        action == Action::Quit
    }

    fn switch_workspace(&self, index: usize) -> Result<(), AppError> {
        let mut ws = self.ws()?;
        ws.switch_to(index)?;
        let area = self.get_work_area()?;
        ws.apply_current_layout(area, 8)
    }

    fn launch_terminal(&self) -> Result<(), AppError> {
        tracing::info!("launching terminal: {}", self.terminal_command);
        let parts: Vec<&str> = self.terminal_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(AppError::Service("terminal command is empty".to_string()));
        }
        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        cmd.spawn()
            .map_err(|e| AppError::Service(format!("failed to launch terminal: {}", e)))?;
        Ok(())
    }
}
