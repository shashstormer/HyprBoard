# HyprBoard

Youtube preview: [https://youtu.be/F0n3Bi8N6v4](https://youtu.be/F0n3Bi8N6v4)

**A Configuration Manager for the Hyprland Ecosystem.**

HyprBoard is a GUI Based Configuration Manager for the Hyprland Ecosystem. Currently it supports 
- Hyprland
- Waybar
- Hyprlock
- Individual presets for all configs and preset bundles creating a group of presets as a theme bundle.

## Features

### Hyprland Management
- **Supports 0.52 and 0.53**
- **Visual Config Editor**: Manage Monitors, Window Rules, Layer Rules, Keybindings, Environment Variables, and Exec commands without touching raw text components.
- **One click Migration**: Migrates 0.52 config to 0.53.
- **Preset System**: Save your working configuration as a preset and switch between different setups instantly.

### Waybar Editor
- **Drag-and-Drop Layout**: Visually reorder modules across Left, Center, and Right sections (using arrow keys and X to disable).
- **Live Style Editor**: Edit `style.css` with a built-in editor and specific UI elements for color picking.
- **Module Configuration**: Configure module settings via a form-based UI or drop into a raw JSON editor for advanced tweaking.
- **Custom Modules**: Easily create and script new custom modules from within the app.

### Hyprlock Designer
- **Form-Based Editing**: Configure General settings, Backgrounds, Input Fields, Labels, Shapes, and Images.
- **Visual Color Picker**: Pick colors for all visual elements intuitively.
- **Preset Support**: Save different lock screen themes as presets.

### Themes & Bundles
- **App Themes**: Switch HyprBoard's own look between Catppuccin flavors (Mocha, Macchiato, Frappe, Latte).
- **Bundles**: Group your active **Hyprland**, **Waybar**, and **Hyprlock** presets into a single **Theme Bundle**. Apply an entire desktop look-and-feel with one click.

### Global Search
- **Fuzzy Search**: Press `Ctrl+K` anywhere to search for any setting, rule, preset, or section across all plugins. Jump straight to the setting you need.

---

## ⌨️ Keybindings

| Keybinding | Action |
|------------|--------|
| `Ctrl + K` | Toggle Global Search / Command Palette |
| `Esc` | Close Search, Modals, or Popups |
| `Enter` | Confirm actions in some modals |

---

### Configuration Paths
HyprBoard expects standard configuration paths. Ensure your dotfiles are located at:
- **Hyprland**: `~/.config/hypr/hyprland.conf`
- **Waybar**: `~/.config/waybar/config.jsonc` (or `config`) and `~/.config/waybar/style.css`
- **Hyprlock**: `~/.config/hypr/hyprlock.conf`

### Auto-Save
All changes are auto saved imediately to disk.

### Progress
- **Waybar**: Still working on it, works and dosent break config files but its not user friendly enough (need suggestions)
- **Wpaperd**: Soon (will be implementing copy files to a wallpapers folder, i want to implement it in a way that you can select certain images only from folders, so you will have duplicate wallpaers when you use this plugin without btrfs filesystem)
- **Import n Export**: Later (If this gains some popularity then, ima look to implement to share configs faster and more easily)
- **Themes**: Curr Catppuccin Mocha, Nord, Drifter supported (dont plan to add anythin else rn, If anyone wants to add any other theme just implement new palette and add it to AppTheme enum in components/theme.rs)
---

### Prerequisites
* **Rust & Cargo** installed on your system.
* **Hyprland** ecosystem installed.

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/shashstormer/hyprboard.git
    cd hyprboard
    ```

2.  **Build and Run:**
    ```bash
    cargo run --release
    ```
    *(Note: Binary releases via GitHub Actions are need to be set up. Contributions are welcome!)*

3.  **Explore:**
    * Use the sidebar to navigate plugins.
    * Press `Ctrl+K` to search for specific settings (e.g., "blur", "opacity").
    * Create a **Bundle** in the Themes tab to save your setup.

---
## Contributing

I just hopped into native app dev with Rust, so contributions are very welcome!
Specifically looking for help setting up a **GitHub Release Workflow** to automate binaries.

---

*Built with Rust & Iced.*
