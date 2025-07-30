# Niri Custom Build - X11 Application Support with Xwayland-Satellite

This is a custom build of the niri Wayland compositor with enhanced X11 application support and Ubuntu 22.04 compatibility.

## 🚀 Features

### Custom Modifications
- **"Short Keys" UI**: Changed hotkey overlay title from "Important Hotkeys" to "Short Keys" for better UX
- **Ubuntu 22.04 Compatibility**: Removed libinput_1_21 feature dependency for compatibility with Ubuntu 22.04's libinput 1.20
- **Xwayland-Satellite Integration**: Built-in configuration for seamless X11 application support

### X11 Application Support
- **Xwayland-Satellite**: Rootless X11 integration for running X11 applications as native Wayland windows
- **Tested Applications**: xterm, xclock, and other X11 applications work seamlessly
- **VNC Support**: X11 applications visible and functional through VNC connections

## 📁 Repository Structure

```
├── src/ui/hotkey_overlay.rs          # Modified: "Short Keys" title
├── src/input/mod.rs                  # Modified: Ubuntu 22.04 libinput compatibility
├── Cargo.toml                        # Modified: Removed libinput_1_21 feature
├── configs/
│   ├── niri-config-with-xwayland-satellite.kdl  # Working niri config
│   └── remote-niri-backup/           # Backup configurations from remote machine
├── niri-wayland-setup.md             # Wayland environment setup guide
├── niri-vnc-setup-complete.md        # VNC setup documentation
├── test-niri-wayland.sh              # Test script for Wayland functionality
└── CUSTOM-README.md                  # This file
```

## 🛠️ Build Instructions

### Prerequisites
```bash
# Install system dependencies
sudo apt update
sudo apt install -y clang libpango1.0-dev libpangocairo-1.0-dev libglib2.0-dev \
    libgtk-3-dev libgdk-pixbuf-2.0-dev libcairo2-dev libxkbcommon-dev libudev-dev \
    libinput-dev libdrm-dev libxcb-composite0-dev libxcb-ewmh-dev libxcb-icccm4-dev \
    libxcb-res0-dev libxcb-xfixes0-dev hwdata meson ninja-build

# Install libdisplay-info from source
git clone https://gitlab.freedesktop.org/emersion/libdisplay-info.git
cd libdisplay-info
meson setup build/
ninja -C build/
sudo ninja -C build/ install
cd ..
```

### Build Niri
```bash
# Clone this repository
git clone https://github.com/walue-ai/niri.git
cd niri
git checkout niri-custom

# Build with Ubuntu 22.04 compatibility
export PKG_CONFIG_PATH="/usr/local/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH"
cargo build --release --no-default-features --features "dbus,systemd"
```

### Build Xwayland-Satellite
```bash
# Clone and build xwayland-satellite
git clone https://github.com/Supreeeme/xwayland-satellite.git
cd xwayland-satellite
cargo build --release

# Install binary
sudo cp target/release/xwayland-satellite /usr/local/bin/
```

## ⚙️ Configuration

### Niri Configuration
Copy the provided configuration:
```bash
mkdir -p ~/.config/niri
cp configs/niri-config-with-xwayland-satellite.kdl ~/.config/niri/config.kdl
```

### Key Configuration Features
- **Xwayland-Satellite Path**: Configured to use `/usr/local/bin/xwayland-satellite`
- **X11 Environment**: `DISPLAY :100` set for X11 applications
- **Hotkey Overlay**: Shows "Short Keys" instead of "Important Hotkeys"

## 🧪 Testing

### VNC Testing
```bash
# Run the provided test script
./test-niri-wayland.sh

# Or follow the VNC setup guide
cat niri-vnc-setup-complete.md
```

### X11 Applications
```bash
# Set display for X11 applications
export DISPLAY=:100  # or the display niri creates

# Test X11 applications
xterm &
xclock &
```

## 📋 Verified Functionality

### Local Environment
- ✅ Niri builds and runs successfully
- ✅ "Short Keys" modification visible in hotkey overlay
- ✅ X11 applications (xterm, xclock) work with xwayland-satellite
- ✅ VNC access with full functionality
- ✅ Ubuntu 22.04 compatibility confirmed

### Remote Deployment
- ✅ Successfully deployed to remote machine (100.111.36.77)
- ✅ Binary transfer and installation completed
- ✅ X11 applications working on remote machine
- ✅ Configuration backup and restore tested

## 🔧 Technical Details

### Modifications Made
1. **Hotkey Overlay** (`src/ui/hotkey_overlay.rs`):
   - Changed `TITLE` constant from "Important Hotkeys" to "Short Keys"

2. **Input Compatibility** (`src/input/mod.rs`):
   - Added conditional compilation for `config_dwtp_set_enabled` function
   - Ensures compatibility with libinput 1.20 (Ubuntu 22.04)

3. **Cargo Configuration** (`Cargo.toml`):
   - Removed `libinput_1_21` feature from input dependency
   - Maintains functionality while ensuring Ubuntu 22.04 compatibility

### Environment Requirements
- **Xwayland**: Version 23.1+ (tested with 23.1.0 and 23.2.6)
- **Libinput**: Version 1.20+ (Ubuntu 22.04 compatible)
- **Mesa**: Version 23.1+ for proper graphics support

## 🚀 Deployment

This custom build has been successfully deployed and tested on:
- **Local Development**: Ubuntu 22.04 with VNC access
- **Remote Production**: Ubuntu 22.04 (100.111.36.77) with full X11 support

## 📞 Support

For issues or questions about this custom build:
1. Check the provided documentation files
2. Verify xwayland-satellite is properly installed and configured
3. Ensure all dependencies are met for your Ubuntu version

## 🎯 Use Cases

Perfect for:
- **Development Environments**: Full X11 application support in Wayland
- **Remote Access**: VNC-compatible setup for remote development
- **Legacy Application Support**: Running X11 applications seamlessly
- **Custom Compositor Needs**: Modified UI and enhanced compatibility

---

**Built with ❤️ by Devin AI for walue-dev**
**Link to Devin run**: https://app.devin.ai/sessions/3af6f33e21634724bbcf36ab6c788c29
