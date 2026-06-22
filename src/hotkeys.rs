use crate::actions::Action;
use crate::error::AppError;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Debug, Clone)]
pub struct HotkeyEntry {
    pub id: i32,
    pub action: Action,
    pub modifiers: u32,
    pub vk: u32,
    pub description: String,
}

pub struct HotkeyRegistry {
    entries: Vec<HotkeyEntry>,
}

impl HotkeyRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register_from_config(
        &mut self,
        hotkeys: &std::collections::HashMap<String, String>,
    ) -> Result<(), AppError> {
        let mut next_id: i32 = 1;

        for (action_name, key_combo) in hotkeys {
            let action = Action::from_str(action_name).ok_or_else(|| {
                AppError::Hotkey(format!("unknown action name: '{}'", action_name))
            })?;

            let (modifiers, vk) = parse_hotkey_string(key_combo)?;

            self.entries.push(HotkeyEntry {
                id: next_id,
                action,
                modifiers,
                vk,
                description: format!("{} -> {}", key_combo, action_name),
            });

            next_id += 1;
        }

        Ok(())
    }

    pub fn entries(&self) -> &[HotkeyEntry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<HotkeyEntry> {
        self.entries
    }

    pub fn add_entry(&mut self, entry: HotkeyEntry) {
        self.entries.push(entry);
    }

    pub fn find_by_id(&self, id: i32) -> Option<Action> {
        self.entries.iter().find(|e| e.id == id).map(|e| e.action)
    }
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_hotkey_string(raw: &str) -> Result<(u32, u32), AppError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::Hotkey("empty hotkey string".to_string()));
    }

    let parts: Vec<&str> = raw.split('+').map(|s| s.trim()).collect();

    let mut modifiers: u32 = 0;
    let mut key = String::new();

    for part in &parts {
        let upper = part.to_uppercase();
        match upper.as_str() {
            "WIN" | "WINDOWS" | "SUPER" => {
                modifiers |= MOD_WIN.0;
            }
            "ALT" | "MENU" => {
                modifiers |= MOD_ALT.0;
            }
            "CTRL" | "CONTROL" => {
                modifiers |= MOD_CONTROL.0;
            }
            "SHIFT" => {
                modifiers |= MOD_SHIFT.0;
            }
            _ => {
                if !key.is_empty() {
                    return Err(AppError::Hotkey(format!(
                        "multiple non-modifier keys in hotkey: '{}'",
                        raw
                    )));
                }
                key = part.to_string();
            }
        }
    }

    if key.is_empty() {
        return Err(AppError::Hotkey(format!(
            "no key specified in hotkey: '{}'",
            raw
        )));
    }

    if modifiers == 0 {
        return Err(AppError::Hotkey(format!(
            "at least one modifier required for hotkey: '{}'",
            raw
        )));
    }

    let vk = str_to_virtual_key(&key)?;

    Ok((modifiers, vk))
}

fn str_to_virtual_key(key: &str) -> Result<u32, AppError> {
    match key.to_uppercase().as_str() {
        "SPACE" | " " => Ok(VK_SPACE.0 as u32),
        "ENTER" | "RETURN" => Ok(VK_RETURN.0 as u32),
        "ESC" | "ESCAPE" => Ok(VK_ESCAPE.0 as u32),
        "TAB" => Ok(VK_TAB.0 as u32),
        "BACK" | "BACKSPACE" => Ok(VK_BACK.0 as u32),
        "DELETE" | "DEL" => Ok(VK_DELETE.0 as u32),
        "INSERT" | "INS" => Ok(VK_INSERT.0 as u32),
        "HOME" => Ok(VK_HOME.0 as u32),
        "END" => Ok(VK_END.0 as u32),
        "PAGEUP" | "PGUP" => Ok(VK_PRIOR.0 as u32),
        "PAGEDOWN" | "PGDN" => Ok(VK_NEXT.0 as u32),
        "UP" => Ok(VK_UP.0 as u32),
        "DOWN" => Ok(VK_DOWN.0 as u32),
        "LEFT" => Ok(VK_LEFT.0 as u32),
        "RIGHT" => Ok(VK_RIGHT.0 as u32),
        "F1" => Ok(VK_F1.0 as u32),
        "F2" => Ok(VK_F2.0 as u32),
        "F3" => Ok(VK_F3.0 as u32),
        "F4" => Ok(VK_F4.0 as u32),
        "F5" => Ok(VK_F5.0 as u32),
        "F6" => Ok(VK_F6.0 as u32),
        "F7" => Ok(VK_F7.0 as u32),
        "F8" => Ok(VK_F8.0 as u32),
        "F9" => Ok(VK_F9.0 as u32),
        "F10" => Ok(VK_F10.0 as u32),
        "F11" => Ok(VK_F11.0 as u32),
        "F12" => Ok(VK_F12.0 as u32),
        c if c.len() == 1 => {
            let ch = c.chars().next().unwrap();
            if ch.is_ascii_alphanumeric() {
                Ok(ch.to_ascii_uppercase() as u32)
            } else {
                Err(AppError::Hotkey(format!("unsupported key: '{}'", key)))
            }
        }
        _ => Err(AppError::Hotkey(format!("unsupported key: '{}'", key))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_win_alt_h() {
        let (mods, vk) = parse_hotkey_string("WIN+ALT+H").unwrap();
        assert_eq!(mods, MOD_WIN.0 | MOD_ALT.0);
        assert_eq!(vk, 'H' as u32);
    }

    #[test]
    fn test_parse_win_alt_shift_j() {
        let (mods, vk) = parse_hotkey_string("WIN+ALT+SHIFT+J").unwrap();
        assert_eq!(mods, MOD_WIN.0 | MOD_ALT.0 | MOD_SHIFT.0);
        assert_eq!(vk, 'J' as u32);
    }

    #[test]
    fn test_parse_win_alt_space() {
        let (mods, vk) = parse_hotkey_string("WIN+ALT+SPACE").unwrap();
        assert_eq!(mods, MOD_WIN.0 | MOD_ALT.0);
        assert_eq!(vk, VK_SPACE.0 as u32);
    }

    #[test]
    fn test_parse_win_alt_enter() {
        let (mods, vk) = parse_hotkey_string("WIN+ALT+ENTER").unwrap();
        assert_eq!(mods, MOD_WIN.0 | MOD_ALT.0);
        assert_eq!(vk, VK_RETURN.0 as u32);
    }

    #[test]
    fn test_parse_win_alt_1() {
        let (mods, vk) = parse_hotkey_string("WIN+ALT+1").unwrap();
        assert_eq!(mods, MOD_WIN.0 | MOD_ALT.0);
        assert_eq!(vk, '1' as u32);
    }

    #[test]
    fn test_parse_ctrl_alt_h() {
        let (mods, vk) = parse_hotkey_string("CTRL+ALT+H").unwrap();
        assert_eq!(mods, MOD_CONTROL.0 | MOD_ALT.0);
        assert_eq!(vk, 'H' as u32);
    }

    #[test]
    fn test_parse_no_modifier_fails() {
        let result = parse_hotkey_string("F");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_fails() {
        let result = parse_hotkey_string("");
        assert!(result.is_err());
    }
}
