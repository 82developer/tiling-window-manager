use crate::window::WindowInfo;

#[derive(Debug, Clone)]
pub struct WindowRule {
    pub class_pattern: Option<String>,
    pub title_pattern: Option<String>,
    pub target_workspace: Option<usize>,
    pub floating: bool,
}

impl WindowRule {
    pub fn matches(&self, window: &WindowInfo) -> bool {
        let class_match = self
            .class_pattern
            .as_ref()
            .map(|p| wildcard_match(&window.class, p))
            .unwrap_or(true);

        let title_match = self
            .title_pattern
            .as_ref()
            .map(|p| wildcard_match(&window.title, p))
            .unwrap_or(true);

        class_match && title_match
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleEngine {
    rules: Vec<WindowRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: WindowRule) {
        self.rules.push(rule);
    }

    pub fn match_window(&self, window: &WindowInfo) -> Option<&WindowRule> {
        self.rules.iter().find(|r| r.matches(window))
    }

    pub fn rules(&self) -> &[WindowRule] {
        &self.rules
    }
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.contains('*') {
        let prefix = pattern.trim_end_matches('*');
        return text.starts_with(prefix);
    }
    text.eq_ignore_ascii_case(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let info = WindowInfo {
            hwnd: 1,
            title: "Notepad".into(),
            class: "Notepad".into(),
            x: 0, y: 0, width: 100, height: 100,
            is_visible: true,
            is_minimized: false,
        };
        let rule = WindowRule {
            class_pattern: Some("Notepad".into()),
            title_pattern: None,
            target_workspace: Some(2),
            floating: false,
        };
        assert!(rule.matches(&info));
    }

    #[test]
    fn test_wildcard_match() {
        let info = WindowInfo {
            hwnd: 1,
            title: "WindowsTerminal.exe".into(),
            class: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
            x: 0, y: 0, width: 100, height: 100,
            is_visible: true,
            is_minimized: false,
        };
        let rule = WindowRule {
            class_pattern: Some("CASCADIA*".into()),
            title_pattern: None,
            target_workspace: Some(1),
            floating: false,
        };
        assert!(rule.matches(&info));
    }

    #[test]
    fn test_no_match() {
        let info = WindowInfo {
            hwnd: 1,
            title: "Chrome".into(),
            class: "Chrome_WidgetWin_1".into(),
            x: 0, y: 0, width: 100, height: 100,
            is_visible: true,
            is_minimized: false,
        };
        let rule = WindowRule {
            class_pattern: Some("Notepad".into()),
            title_pattern: None,
            target_workspace: Some(2),
            floating: false,
        };
        assert!(!rule.matches(&info));
    }
}
