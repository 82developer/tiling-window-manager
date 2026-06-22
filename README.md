# rust-tiling-window-manager

A keyboard-driven tiling window manager for Windows 11, inspired by i3wm, bspwm, sway, and yabai. Written in Rust with zero external runtime dependencies beyond the Windows API.

**Current version: 0.1.0** - Hotkey-driven window control with layout engine, monitor detection, workspace support, and terminal launcher.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Installation](#installation)
- [Building](#building)
- [Running](#running)
- [Configuration](#configuration)
- [Hotkey Reference](#hotkey-reference)
- [Kinesis Advantage2 Integration](#kinesis-advantage2-integration)
- [Troubleshooting](#troubleshooting)
- [Windows Limitations](#windows-limitations)
- [Roadmap](#roadmap)
- [License](#license)

## Overview

rust-tiling-window-manager (rtwm) is a background process that registers global hotkeys and controls existing windows via the Win32 API. It does **not** replace the Windows shell or modify system components. It works alongside Explorer, the taskbar, and all standard Windows applications.

Key features:
- **Global hotkeys** via `RegisterHotKey` - works in any application
- **Window movement** - resize and reposition windows with keyboard
- **Layout engine** - left/right/top/bottom/fullscreen/centered layouts
- **Monitor-aware** - all calculations relative to the active monitor's work area
- **Configurable** - every keybinding defined in `config.toml`
- **Action-based** - keys are mapped to abstract actions, not Win32 calls directly

## Architecture

```
┌──────────────────────────────┐
│         main.rs              │  Entry point, tracing init
├──────────────────────────────┤
│  Application Layer           │
│  app.rs      config.rs       │  Orchestration, config parsing
│  commands.rs hotkeys.rs      │  Command dispatch, hotkey parsing
│  error.rs                    │  Error types
├──────────────────────────────┤
│  Services Layer              │
│  window_service.rs           │  Window operations + filtering
│  monitor_service.rs          │  Monitor detection
│  layout_service.rs           │  Layout application
│  workspace_service.rs        │  Workspace management
├──────────────────────────────┤
│  Domain Layer                │
│  actions.rs  layout.rs       │  Pure types, layout math
│  window.rs   monitor.rs      │  Domain models
│  workspace.rs                │  Workspace model
├──────────────────────────────┤
│  Infrastructure Layer        │
│  win32/window_api.rs         │  Win32 FFI (window ops)
│  win32/monitor_api.rs        │  Win32 FFI (monitor ops)
│  win32/hotkey_api.rs         │  Win32 FFI (hotkey registration)
└──────────────────────────────┘
```

### Design Principles

- **Clean Architecture**: Domain logic has zero knowledge of Win32. Infrastructure layer is the only place with `unsafe` and Win32 calls.
- **SOLID**: Single responsibility per module. Dependency injection from config through services.
- **KISS**: No over-engineering. Concrete types where traits aren't needed yet. Simple flow.

### Execution Flow

```
1. main.rs loads config.toml
2. App::new() creates all services
3. HotkeyRegistry parses keybindings → (modifiers, virtual key)
4. Win32HotkeyApi registers each hotkey via RegisterHotKey
5. GetMessageW loop blocks waiting for WM_HOTKEY
6. WM_HOTKEY → ID lookup → Action
7. Action → CommandExecutor → Service → Infrastructure → Win32 API
8. On quit hotkey: PostQuitMessage(0) → exit loop → shutdown
```

## Installation

### Prerequisites

- Windows 11 (x64)
- Rust Stable 1.75+ (install via [rustup.rs](https://rustup.rs))
- No other dependencies required

### From Source

```powershell
git clone https://github.com/user/rust-tiling-window-manager.git
cd rust-tiling-window-manager
cargo build --release
```

The binary will be at `target/release/rtwm.exe`.

## Building

```powershell
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run
```

## Running

```powershell
# Run with default config (config.toml in current directory)
cargo run

# Run with specific config file
cargo run -- /path/to/custom-config.toml

# Run release binary
.\target\release\rtwm.exe
.\target\release\rtwm.exe my-config.toml
```

The application runs in the foreground as a console process. It registers global hotkeys and enters a message loop waiting for hotkey events. Press `Ctrl+C` to stop, or use a configured quit hotkey.

### Auto-start

Add to Windows startup (optional):
1. Press `Win+R`, type `shell:startup`
2. Create a shortcut to `rtwm.exe` with the config path as argument
3. Or use Task Scheduler to run at login

## Configuration

Configuration is in `config.toml`. The application searches for this file in the current directory by default, or takes a path as the first command-line argument.

### Complete Example

```toml
[layout]
gap = 8       # Gap between windows in pixels
margin = 0    # Margin from screen edge in pixels

[terminal]
command = "wt.exe"  # Command to launch terminal

[ignore]
classes = [
    "Shell_TrayWnd",
    "Progman",
    "Windows.UI.Core.CoreWindow",
    "ApplicationFrameWindow",
]
titles = [
    "Program Manager",
]

[hotkeys]
move_left  = "WIN+ALT+H"
move_down  = "WIN+ALT+J"
move_up    = "WIN+ALT+K"
move_right = "WIN+ALT+L"

focus_left  = "WIN+ALT+SHIFT+H"
focus_down  = "WIN+ALT+SHIFT+J"
focus_up    = "WIN+ALT+SHIFT+K"
focus_right = "WIN+ALT+SHIFT+L"

fullscreen = "WIN+ALT+F"
center     = "WIN+ALT+SPACE"

launch_terminal = "WIN+ALT+ENTER"

next_workspace     = "WIN+ALT+N"
previous_workspace = "WIN+ALT+P"

workspace_1 = "WIN+ALT+1"
workspace_2 = "WIN+ALT+2"
workspace_3 = "WIN+ALT+3"
workspace_4 = "WIN+ALT+4"
workspace_5 = "WIN+ALT+5"

move_to_workspace_1 = "WIN+ALT+SHIFT+1"
move_to_workspace_2 = "WIN+ALT+SHIFT+2"
move_to_workspace_3 = "WIN+ALT+SHIFT+3"
move_to_workspace_4 = "WIN+ALT+SHIFT+4"
move_to_workspace_5 = "WIN+ALT+SHIFT+5"

layout_left_half   = "WIN+ALT+Q"
layout_right_half  = "WIN+ALT+W"
layout_top_half    = "WIN+ALT+E"
layout_bottom_half = "WIN+ALT+R"

quit = "WIN+ALT+ESC"
```

### Layout Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `layout.gap` | 8 | Gap between windows in pixels |
| `layout.margin` | 0 | Margin from monitor edges |

### Terminal Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `terminal.command` | `"wt.exe"` | Command to launch when terminal hotkey is pressed |

### Ignore Configuration

Windows that match these classes or titles are excluded from window operations:

```toml
[ignore]
classes = ["Shell_TrayWnd", "Progman"]
titles = ["Program Manager"]
```

### Hotkey Format

Hotkeys are specified as `MODIFIER+MODIFIER+KEY`:

**Modifiers:**
- `WIN` / `WINDOWS` / `SUPER`
- `ALT` / `MENU`
- `CTRL` / `CONTROL`
- `SHIFT`

**Keys:**
- Single characters: `A` through `Z`, `0` through `9`
- Special: `SPACE`, `ENTER`, `RETURN`, `ESC`, `ESCAPE`, `TAB`, `BACKSPACE`, `DELETE`
- Navigation: `UP`, `DOWN`, `LEFT`, `RIGHT`, `HOME`, `END`, `PAGEUP`, `PAGEDOWN`
- Function: `F1` through `F12`

**At least one modifier is required** for each hotkey.

Examples:
```toml
fullscreen = "WIN+ALT+F"
close      = "CTRL+ALT+X"
reload     = "WIN+SHIFT+R"
```

### Available Actions

| Action | Description |
|--------|-------------|
| `move_left` | Move window 80px left |
| `move_down` | Move window 80px down |
| `move_up` | Move window 80px up |
| `move_right` | Move window 80px right |
| `focus_left` | Focus window to the left |
| `focus_down` | Focus window below |
| `focus_up` | Focus window above |
| `focus_right` | Focus window to the right |
| `fullscreen` | Maximize window to work area |
| `center` | Center window at 75% size |
| `launch_terminal` | Launch configured terminal |
| `next_workspace` | Switch to next workspace |
| `previous_workspace` | Switch to previous workspace |
| `workspace_1` through `workspace_5` | Switch to workspace N |
| `move_to_workspace_1` through `move_to_workspace_5` | Move window to workspace N |
| `layout_left_half` | Toggle left half layout |
| `layout_right_half` | Toggle right half layout |
| `layout_top_half` | Toggle top half layout |
| `layout_bottom_half` | Toggle bottom half layout |
| `quit` | Exit the application |

### Changing the Mod Key

The default mod key is `WIN+ALT`. To change it, update all hotkey entries in `config.toml`:

```toml
# Using CTRL+ALT as mod key
move_left  = "CTRL+ALT+H"
move_down  = "CTRL+ALT+J"
# ... etc
```

Or use the Kinesis Advantage2 firmware to remap a thumb key to emit `WIN+ALT`.

## Hotkey Reference

### Default Keybindings (optimized for Kinesis Advantage2)

**Window Movement:**
| Key | Action |
|-----|--------|
| `Win+Alt+H` | Move window left |
| `Win+Alt+J` | Move window down |
| `Win+Alt+K` | Move window up |
| `Win+Alt+L` | Move window right |
| `Win+Alt+F` | Fullscreen |
| `Win+Alt+Space` | Center window |

**Focus:**
| Key | Action |
|-----|--------|
| `Win+Alt+Shift+H` | Focus left |
| `Win+Alt+Shift+J` | Focus down |
| `Win+Alt+Shift+K` | Focus up |
| `Win+Alt+Shift+L` | Focus right |

**Layouts:**
| Key | Action |
|-----|--------|
| `Win+Alt+Q` | Left half |
| `Win+Alt+W` | Right half |
| `Win+Alt+E` | Top half |
| `Win+Alt+R` | Bottom half |

**Workspaces:**
| Key | Action |
|-----|--------|
| `Win+Alt+N` | Next workspace |
| `Win+Alt+P` | Previous workspace |
| `Win+Alt+1`-`5` | Switch to workspace 1-5 |
| `Win+Alt+Shift+1`-`5` | Move window to workspace 1-5 |

**Application:**
| Key | Action |
|-----|--------|
| `Win+Alt+Enter` | Launch terminal |
| `Win+Alt+Esc` | Quit |

## Kinesis Advantage2 Integration

The default keybindings are specifically designed for the Kinesis Advantage2 keyboard:

### Ergonomic Design

- **Thumb clusters**: `WIN` and `ALT` are positioned near the thumb clusters. Keep thumbs on these modifiers while navigating with home-row fingers.
- **Home-row navigation**: `H`/`J`/`K`/`L` mirrors Vim/Neovim navigation. No hand repositioning needed.
- **Workspace switching**: `N`/`P` (next/previous) are accessible without leaving home row. Number keys `1`-`5` for direct access.
- **Layout keys**: `Q`/`W`/`E`/`R` form a row for layout operations, muscle-memory friendly.

### SmartSet Configuration

For advanced users with Kinesis SmartSet:

1. **Remap Caps Lock to a layer toggle** that emits `Win+Alt` on your preferred thumb key
2. **Create a "tiling" layer** where single keys emit `Win+Alt+{key}` combinations
3. **Map the left thumb cluster** to `Win` and `Alt` for chord-based activation

### Customizing for Your Kinesis Layout

Edit `config.toml` to match your SmartSet configuration. The architecture supports any modifier combination, so you can adapt the keybindings to whatever your Kinesis firmware emits.

## Troubleshooting

### Hotkeys not working

1. **Check for conflicts**: Windows, other applications (AutoHotkey, gaming software) may register conflicting hotkeys. Use `WIN+ALT+ESC` to test if the application is receiving hotkeys at all.
2. **Run as administrator**: Some hotkey combinations require elevated privileges.
3. **Verify logging**: Run with `RUST_LOG=debug cargo run` to see which hotkeys are registered and when they fire.

### Window doesn't move

1. **Check if window is ignored**: Verify the window's class and title aren't in the `[ignore]` section of `config.toml`.
2. **Minimized windows**: The application automatically restores minimized windows before moving them.
3. **System windows**: Some Windows system windows (Task Manager, UAC dialogs) cannot be moved by external processes.

### Cannot focus a window

`SetForegroundWindow` has restrictions on Windows 10/11. The foreground can only be changed by:
- The process that currently owns the foreground
- Processes that received input recently
- Processes started by the foreground process

If `SetForegroundWindow` fails, the application logs a warning but continues. This is a Windows security limitation.

### Configuration not loading

Run with `RUST_LOG=info cargo run` to see which config file is being loaded. The default path is `config.toml` in the current working directory.

### Terminal not launching

Verify the terminal command in `config.toml`:
```toml
[terminal]
command = "wt.exe"   # Windows Terminal
# or
command = "powershell.exe"
# or
command = "cmd.exe"
```

## Windows Limitations

### What this can do
- Move and resize existing windows
- Register global hotkeys
- Detect monitors and their work areas
- Launch applications
- Track workspaces conceptually

### What this cannot do (by design)
- **Replace the Windows shell** - Explorer.exe still manages the desktop, taskbar, and system tray
- **Create virtual desktops** - Windows 10/11 has its own virtual desktop API; rtwm manages its own workspace concept
- **Draw window decorations** - Windows maintains its own title bars and borders
- **Intercept all keyboard input** - Only registered global hotkeys are captured; typing in applications is unaffected
- **Control UWP/app store windows reliably** - Some modern Windows apps use different window management APIs

### Focus restrictions
Windows 10/11 restricts which processes can call `SetForegroundWindow`. The application will attempt to focus windows but may fail silently. Use `Alt+Tab` as a fallback.

### No compositor access
Unlike i3/sway on Linux which work at the display server level, rtwm works at the Win32 API level. This means:
- No frame-perfect window animations
- No custom window decorations
- No interception of window creation events
- Windows can still overlap (no enforced tiling)

## Roadmap

### v0.1.0 (Current)
- [x] Global hotkey registration
- [x] Window movement (up/down/left/right)
- [x] Layout engine (half/fullscreen/centered)
- [x] Monitor detection
- [x] Configuration via TOML
- [x] Terminal launcher
- [x] Error handling with tracing
- [x] Unit tests for layouts, config parsing, hotkey parsing

### v0.2.0 (Planned)
- [ ] Window tree layout (split containers like i3)
- [ ] Monocle layout (single window fullscreen, others hidden)
- [ ] Stacking layout
- [ ] Window rules by class/title (auto-assign to workspace/layout)
- [ ] Proper focus navigation (find adjacent window spatially)

### v0.3.0
- [ ] Multiple monitor support with per-monitor workspaces
- [ ] Workspace show/hide (actually hide windows not in current workspace)
- [ ] Session persistence (save/restore window positions)
- [ ] Auto-start configuration

### v0.4.0
- [ ] Status bar (configurable, per-monitor)
- [ ] System tray integration
- [ ] Scratchpad support
- [ ] Floating window support
- [ ] IPC interface for external control

### Future
- [ ] Integration with Neovim via RPC
- [ ] Integration with Windows Terminal profiles
- [ ] Configuration hot-reload
- [ ] GUI configuration tool (optional, not required)

## License

MIT License - see LICENSE file for details.

---

Built with Rust, the [windows](https://crates.io/crates/windows) crate, and a lot of inspiration from i3wm.
