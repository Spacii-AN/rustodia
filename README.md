# Exodia Contagion Macro for Warframe (Rust - Optimized)

A high-performance macro automation tool for Warframe's Exodia Contagion combo, written in Rust for maximum speed and efficiency.

**Made by Spacii-AN**

## 🚀 Performance

This Rust implementation is optimized for maximum performance:

- **10-50x lower latency** - Near-instant input response
- **5-10x lower CPU usage** - Minimal system impact
- **50-100x lower memory footprint** - Efficient resource usage
- **Native performance** - Compiled to machine code

## Features

- **Exodia Contagion Sequence**: Automated execution of the complete combo sequence
- **Hold-to-Activate**: Macro only runs while the side mouse button is held down
- **Rapid Click Macro**: Quick burst click function
- **Cross-Platform**: Supports macOS, Windows, and Linux
- **Window Detection**: Automatically stops when Warframe loses focus
- **GUI Configuration**: Simple graphical interface to adjust timings and keybinds (run with `--gui` flag)
- **Highly Configurable**: All keybinds and timing settings easily adjustable via GUI or code
- **High Precision Timing**: Sub-millisecond accuracy using optimized timing

## Warframe Settings

**IMPORTANT**: Before using the macro, ensure this setting is OFF in Warframe:
- `Melee with Fire Weapon Input` → **OFF**

## Quick Start

### Prerequisites

- Rust toolchain (install from https://rustup.rs/)
- Platform-specific dependencies:
  - **Linux**: X11 development libraries (`libx11-dev`, `libxdo-dev` for xdotool)
  - **Windows**: Windows SDK
  - **macOS**: Xcode Command Line Tools

### Build & Run

```bash
# Build the optimized version
cargo build --release

# Run the macro (CLI mode)
cargo run --release

# Run with GUI for easy configuration
cargo run --release -- --gui
# or
cargo run --release -- -g
```

The optimized binary will be in `target/release/pt-macro` (or `target/release/pt-macro.exe` on Windows).

### GUI Mode

The GUI provides an easy way to adjust all timing and keybind settings without editing code:

- **Timing Settings**: Adjust FPS, delays, and timing values with sliders
- **Keybind Settings**: Click "Set" buttons and press the desired key/button to configure
- **Live Updates**: Changes apply immediately to new macro sequences
- **Cross-Platform**: Works on Windows, Linux, and macOS
- **Easy Keybinding**: No need to know key names - just click and press!

**Keybinding Instructions:**
1. Click the "Set" button next to any keybind
2. Press the key or mouse button you want to use
3. The keybind is automatically detected and set
4. Press Escape to cancel if you change your mind

Simply run with the `--gui` or `-g` flag to open the configuration window.

## Configuration

### GUI Mode (Recommended)

Run with `--gui` flag to open a graphical configuration window:
```bash
cargo run --release -- --gui
```

The GUI allows you to adjust all settings with sliders and text inputs. Changes apply immediately to new macro sequences.

### Manual Configuration

Alternatively, configuration can be done in `src/config.rs`:

- **Keybinds**: Edit the `SharedConfig::default()` implementation
- **Timing**: Modify timing values in the `SharedConfig` struct

All settings are clearly organized for easy customization.

## Controls

- **Hold Side Mouse Button**: Activates the Exodia Contagion sequence
- **Press 'j'**: Performs rapid clicks
- **Press F11**: Toggles all macros on/off
- **Ctrl+C**: Exit

## Troubleshooting

### Linking Errors

If you encounter linking errors, ensure you have the required system libraries installed:

**Linux:**
```bash
# Debian/Ubuntu
sudo apt install libx11-dev libxdo-dev

# Arch Linux
sudo pacman -S libx11 xdotool

# Fedora
sudo dnf install libX11-devel xdotool
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Ensure you have Visual Studio Build Tools or Visual Studio installed

### Permission Issues

**Linux**: You may need to be in the `input` group:
```bash
sudo usermod -a -G input $USER
# Log out and back in
```

**macOS**: Grant accessibility permissions in System Preferences → Security & Privacy → Privacy → Accessibility

## License

This project is provided as-is for personal use. Use at your own risk.

## Disclaimer

This macro is for educational purposes. Ensure that using macros complies with Warframe's Terms of Service and your local regulations. The authors are not responsible for any consequences resulting from the use of this software.
