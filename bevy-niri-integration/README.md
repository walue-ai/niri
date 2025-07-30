# Bevy-Niri Integration

A high-performance integration between the Niri Wayland compositor and Bevy game engine, enabling real-time display of multiple Niri screen outputs within a single Bevy application.

## Features

- **Multi-Screen Support**: Display multiple Niri compositor outputs simultaneously in one Bevy scene
- **High Performance**: Achieves 60+ FPS with optimized texture streaming
- **Hybrid Buffer Support**: Automatic selection between DMA and SHM buffers for optimal performance
- **Adaptive Performance**: Real-time performance monitoring and buffer method selection
- **Zero-Copy GPU Transfers**: DMA buffer support for minimal latency
- **Hybrid Configuration**: Both KDL config file and Rust API configuration support

## Quick Start

```rust
use bevy::prelude::*;
use bevy_niri_integration::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NiriCapturePlugin {
            outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
            capture_fps: 60.0,
            prefer_dmabuf: true,
        })
        .run();
}
```

## Architecture

The integration consists of several key components:

1. **Wayland Screencopy Client**: Captures screen content from Niri using the screencopy protocol
2. **Bevy Plugin System**: Integrates captured content into Bevy's rendering pipeline
3. **Multi-Screen Display**: Renders multiple outputs as separate textured meshes
4. **DMA Buffer Integration**: Zero-copy GPU texture import for high performance
5. **Adaptive Performance System**: Automatic optimization based on runtime performance

## Performance

- **Target**: 60+ FPS for real-time screen capture and display
- **Latency**: <2ms with DMA buffers, <10ms with SHM fallback
- **Memory**: Efficient zero-copy transfers when GPU supports DMA buffers
- **CPU Usage**: <5% with DMA, <15% with SHM buffers

## Configuration

### KDL Configuration (config.kdl)

```kdl
bevy-capture {
    capture-fps = 60
    prefer-dmabuf = true
    damage-tracking = true
    adaptive-performance = true
    max-memory-usage = 0.8
}
```

### Rust API Configuration

```rust
NiriCapturePlugin {
    outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
    capture_fps: 60.0,
    prefer_dmabuf: true,
}
```

## Examples

Run the multi-screen display example:

```bash
cargo run --example multi_screen_display
```

## Testing

Run unit tests:
```bash
cargo test
```

Run integration tests:
```bash
cargo test --test integration_tests
```

Run performance benchmarks:
```bash
cargo bench
```

## Requirements

- Niri Wayland compositor with screencopy protocol support
- Bevy 0.14+
- Wayland development libraries
- GPU with DMA buffer support (optional, falls back to SHM)

## License

GPL-3.0-or-later
