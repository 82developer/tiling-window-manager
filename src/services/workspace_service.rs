use crate::error::AppError;
use crate::layout::Rect;
use crate::rules::{RuleEngine, WindowRule};
use crate::services::window_service::WindowService;
use crate::tree::{ContainerTree, FocusDirection, Layout};

pub struct WorkspaceService {
    window_service: WindowService,
    trees: Vec<ContainerTree>,
    current_index: usize,
    rule_engine: RuleEngine,
    workspace_window_map: Vec<Vec<isize>>,
}

impl WorkspaceService {
    pub fn new(window_service: WindowService, workspace_count: usize) -> Self {
        let trees: Vec<ContainerTree> = (0..workspace_count).map(|_| ContainerTree::new()).collect();

        Self {
            window_service,
            trees,
            current_index: 0,
            rule_engine: RuleEngine::new(),
            workspace_window_map: vec![Vec::new(); workspace_count],
        }
    }

    pub fn current_tree(&self) -> &ContainerTree {
        &self.trees[self.current_index]
    }

    pub fn current_tree_mut(&mut self) -> &mut ContainerTree {
        &mut self.trees[self.current_index]
    }

    pub fn add_rule(&mut self, rule: WindowRule) {
        self.rule_engine.add_rule(rule);
    }

    pub fn reset(&mut self) -> Result<(), AppError> {
        tracing::info!("resetting all workspaces to initial state");

        for hwnd_list in &self.workspace_window_map {
            for hwnd in hwnd_list {
                let _ = self.window_service.show_window(*hwnd, true);
            }
        }

        self.trees = (0..self.trees.len()).map(|_| ContainerTree::new()).collect();
        self.workspace_window_map = vec![Vec::new(); self.trees.len()];
        self.current_index = 0;

        let windows = self.window_service.enumerate_visible_windows()?;
        tracing::info!("re-detected {} visible windows", windows.len());

        for w in &windows {
            let hwnd = w.hwnd;
            if let Some(rule) = self.rule_engine.match_window(w) {
                if let Some(target_ws) = rule.target_workspace {
                    if target_ws < self.trees.len() {
                        self.trees[target_ws].add_window(hwnd);
                        self.workspace_window_map[target_ws].push(hwnd);
                        continue;
                    }
                }
            }
            self.trees[0].add_window(hwnd);
            self.workspace_window_map[0].push(hwnd);
        }

        let count = self.trees[0].window_count();
        tracing::info!("workspace 1 has {} windows", count);
        Ok(())
    }

    pub fn focus_window(&self, hwnd: isize) -> Result<(), AppError> {
        self.window_service.focus_window(hwnd)
    }

    /// Sync internal focus with the real foreground window (e.g. after mouse click)
    pub fn sync_focus(&mut self) {
        let hwnd = self.window_service.get_foreground_hwnd();
        if hwnd == 0 {
            return;
        }
        if let Some(id) = self.trees[self.current_index].find_window(hwnd) {
            if self.trees[self.current_index].focused_id() != id {
                self.trees[self.current_index].focus_container(id);
                tracing::debug!("focus synced to hwnd={}", hwnd);
            }
        }
    }

    pub fn toggle_split(&mut self) {
        self.current_tree_mut().toggle_split();
    }

    pub fn set_split_horizontal(&mut self) {
        self.current_tree_mut().set_split_horizontal();
    }

    pub fn set_split_vertical(&mut self) {
        self.current_tree_mut().set_split_vertical();
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.current_tree_mut().set_layout(layout);
    }

    pub fn switch_to(&mut self, index: usize) -> Result<(), AppError> {
        if index >= self.trees.len() {
            return Err(AppError::Service(format!("workspace index {} out of bounds", index)));
        }
        if index == self.current_index {
            return Ok(());
        }
        let old = self.current_index;
        self.current_index = index;
        self.apply_visibility(old, index)?;
        tracing::info!("switched workspace: {} -> {}", old + 1, index + 1);
        Ok(())
    }

    pub fn next_workspace(&mut self) -> Result<(), AppError> {
        let next = (self.current_index + 1) % self.trees.len();
        self.switch_to(next)
    }

    pub fn previous_workspace(&mut self) -> Result<(), AppError> {
        let prev = if self.current_index == 0 {
            self.trees.len() - 1
        } else {
            self.current_index - 1
        };
        self.switch_to(prev)
    }

    fn apply_visibility(&mut self, old_idx: usize, new_idx: usize) -> Result<(), AppError> {
        let old_windows = self.workspace_window_map[old_idx].clone();
        for hwnd in &old_windows {
            let _ = self.window_service.show_window(*hwnd, false);
        }
        let new_windows = self.workspace_window_map[new_idx].clone();
        for hwnd in &new_windows {
            let _ = self.window_service.show_window(*hwnd, true);
        }
        Ok(())
    }

    pub fn add_window_to_current(&mut self, hwnd: isize) {
        self.current_tree_mut().add_window(hwnd);
        if !self.workspace_window_map[self.current_index].contains(&hwnd) {
            self.workspace_window_map[self.current_index].push(hwnd);
        }
    }

    pub fn remove_window_from_current(&mut self, hwnd: isize) {
        self.current_tree_mut().remove_window(hwnd);
        self.workspace_window_map[self.current_index].retain(|h| *h != hwnd);
    }

    pub fn detect_and_manage_window(&mut self) -> Result<(), AppError> {
        let hwnd = self.window_service.get_foreground_hwnd();
        if hwnd == 0 {
            return Ok(());
        }

        let window_info = match self.window_service.get_foreground_window() {
            Ok(info) => info,
            Err(_) => return Ok(()),
        };

        if let Some(rule) = self.rule_engine.match_window(&window_info) {
            if let Some(target_ws) = rule.target_workspace {
                if target_ws != self.current_index && target_ws < self.trees.len() {
                    self.window_service.show_window(hwnd, false).ok();
                    self.trees[target_ws].add_window(hwnd);
                    self.workspace_window_map[target_ws].push(hwnd);
                    tracing::info!(
                        "auto-assigned '{}' to workspace {} per rule",
                        window_info.title, target_ws + 1
                    );
                    return Ok(());
                }
            }
        }

        if self.trees[self.current_index].find_window(hwnd).is_none() {
            self.add_window_to_current(hwnd);
            let count = self.trees[self.current_index].window_count();
            tracing::info!(
                "window '{}' (class={}) added to workspace {} ({} windows total)",
                window_info.title,
                window_info.class,
                self.current_index + 1,
                count
            );
        } else {
            tracing::info!(
                "window '{}' already in workspace {}",
                window_info.title,
                self.current_index + 1
            );
        }

        Ok(())
    }

    pub fn move_focused(&mut self, dir: &FocusDirection, monitor_area: Rect) -> bool {
        self.current_tree_mut().move_focused_in_direction(dir, monitor_area)
    }

    pub fn apply_current_layout(&self, monitor_area: Rect, gap: i32) -> Result<(), AppError> {
        let tree = self.current_tree();
        let layouts = tree.calculate_layouts(monitor_area, gap);
        tracing::info!(
            "applying layout: {} windows, area=({},{},{},{}), direction={:?}",
            layouts.len(),
            monitor_area.x, monitor_area.y, monitor_area.width, monitor_area.height,
            tree.get_split_direction(tree.root_id())
        );
        for (hwnd, rect) in layouts {
            tracing::info!(
                "  window hwnd={} → ({},{},{},{})",
                hwnd, rect.x, rect.y, rect.width, rect.height
            );
            if let Err(e) = self.window_service.move_window(hwnd, rect.x, rect.y, rect.width, rect.height) {
                tracing::error!("failed to move window hwnd={}: {}", hwnd, e);
            }
        }
        Ok(())
    }
}
