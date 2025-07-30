# Niri Wayland Environment Setup on Ubuntu 22.04

## Components Installed
- Niri compositor: v25.05.1 (built from source with Ubuntu 22.04 compatibility)
- Waypipe: v0.8.2 (system package - 0.10.4 build failed due to FFmpeg compatibility)
- Mesa: v23.2.1 (system package - meets 23.1+ requirement)
- Weston: v9.0.0 (reference Wayland compositor)
- Sway: v1.7 (alternative Wayland compositor)

## Usage
1. Start Wayland session: `weston --width=1200 --height=800 &`
2. Set environment: `export WAYLAND_DISPLAY=wayland-1`
3. Run niri: `cd ~/repos/niri && ./target/release/niri`
4. Test waypipe: `waypipe --version`

## Compatibility Notes
- Modified niri for libinput 1.20.0 compatibility
- Waypipe 0.10.4 build failed on Ubuntu 22.04 due to FFmpeg version requirements
- Mesa 23.2.1 provides required 23.1+ support
- Ubuntu 22.04 FFmpeg (4.4.2) vs waypipe 0.10.4 requirement (58.11.100+)

## Alternative Test Methods
- Use sway: `sway` (starts full Wayland session)
- Use nested weston: `weston --width=1200 --height=800`
- Test script: `~/test-niri-wayland.sh`
