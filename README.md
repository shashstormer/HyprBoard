# HyprBoard

![Rust](https://img.shields.io/badge/Made_with-Rust-orange?logo=rust)
![Hyprland](https://img.shields.io/badge/Config-Hyprland-blue)

[**Youtube Preview**](https://youtu.be/F0n3Bi8N6v4)

**A Configuration Manager for the Hyprland Ecosystem.**

HyprBoard is a GUI-based Configuration Manager built with **Rust** and **Iced**. It simplifies the management of your Hyprland ecosystem by providing a unified interface for:
* **Hyprland**
* **Waybar**
* **Hyprlock**

It features individual presets for all configurations and "Theme Bundles" to group presets into a cohesive desktop look.

---

## Features

### Hyprland Management
* **Version Support:** Support for Hyprland **0.52** and **0.53**.
* **Visual Config Editor:** Manage Monitors, Window Rules, Layer Rules, Keybindings, Environment Variables, and Exec commands without editing raw text files.
* **Seamless Migration:** One-click migration tool to upgrade 0.52 configs to 0.53 standards.
* **Preset System:** Snapshot your working configuration as a preset and switch between setups instantly.

### Waybar Editor
* **Drag-and-Drop Layout:** Visually reorder modules across Left, Center, and Right sections (use arrow keys to move, `X` to disable).
* **Live Style Editor:** Edit `style.css` with a built-in editor featuring UI-based color pickers.
* **Module Configuration:** Configure settings via a form-based UI or drop into a raw JSON editor for advanced tweaking.
* **Custom Modules:** Easily create and script new custom modules directly within the app.

### Hyprlock Designer
* **Form-Based Editing:** Configure General settings, Backgrounds, Input Fields, Labels, Shapes, and Images.
* **Visual Color Picker:** Intuitively pick colors for all visual elements.
* **Preset Support:** Save and switch between different lock screen themes.

### Themes & Bundles
* **App Themes:** Toggle HyprBoard's internal UI between Catppuccin flavors (Mocha, Macchiato, Frappe, Latte), Nord, and Drifter.
* **Bundles:** Group your active **Hyprland**, **Waybar**, and **Hyprlock** presets into a single **Theme Bundle**. Apply an entire desktop look-and-feel with one click.

### Global Search
* **Fuzzy Search:** Press `Ctrl+K` anywhere to search for any setting, rule, preset, or section across all plugins. Jump straight to the setting you need.

---

## ⌨️ Keybindings

| Keybinding | Action |
| :--- | :--- |
| `Ctrl + K` | Toggle Global Search / Command Palette |
| `Esc` | Close Search, Modals, or Popups |
| `Enter` | Confirm actions in modals |

---

## Configuration & Behavior

### Configuration Paths
HyprBoard expects standard configuration paths. Ensure your dotfiles are located at:
* **Hyprland:** `~/.config/hypr/hyprland.conf`
* **Waybar:** `~/.config/waybar/config.jsonc` (or `config`) and `~/.config/waybar/style.css`
* **Hyprlock:** `~/.config/hypr/hyprlock.conf`

### Auto-Save
All changes are persisted to disk **immediately** to ensure no data loss.

### Safe Migration
The app automatically detects Hyprland versions. If it detects a 0.52 config running on 0.53, it will present a modal offering to safely migrate your configuration.

---

## Roadmap & Status

* **Waybar:** Functional and safe for config files, but currently undergoing UX refinements to improve user-friendliness. Suggestions are welcome.
* **Wpaperd:** Coming soon. Will implement file management (copying files to a dedicated wallpapers folder). *Note: Without Btrfs, this may result in duplicate wallpaper files.*
* **Import & Export:** Planned for a future release to facilitate easier community sharing.
* **Themes:** Currently supports Catppuccin Mocha, Nord, and Drifter. New themes can be added by implementing a new palette in `components/theme.rs`.

---

## Installation

### Prerequisites
* **Rust & Cargo** installed on your system.
* **Hyprland** ecosystem installed.

### Build from Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/shashstormer/hyprboard.git
    cd hyprboard
    ```

2.  **Build and Run:**
    ```bash
    cargo run --release
    ```

---

## Contributing

I am new to native app development with Rust, so contributions are very welcome!

**Help Needed:** I am specifically looking for assistance in setting up a **GitHub Release Workflow** to automate binary releases.

*Built with Rust & Iced.*
