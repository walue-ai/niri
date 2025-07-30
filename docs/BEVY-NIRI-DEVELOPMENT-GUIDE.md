# Bevy-Niri Integration Development Guide

## 🚀 Getting Started

This guide provides step-by-step instructions for developing the Bevy-Niri integration system, including code examples, testing procedures, and troubleshooting tips.

## 📋 Prerequisites

### **Development Environment Setup**

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies
sudo apt update
sudo apt install -y \
    libwayland-dev \
    libxkbcommon-dev \
    libegl1-mesa-dev \
    libdrm-dev \
    libgbm-dev \
    pkg-config \
    clang

# Install Niri (custom build with X11 support)
git clone https://github.com/walue-ai/niri.git
cd niri
git checkout niri-custom
cargo build --release --no-default-features --features "dbus,systemd"
```

### **Bevy Project Setup**

```bash
# Create new Bevy project
cargo new bevy-niri-integration
cd bevy-niri-integration

# Add dependencies to Cargo.toml
```

```toml
[dependencies]
bevy = "0.14"
wayland-client = "0.31"
wayland-protocols-wlr = "0.2"
smithay-client-toolkit = "0.18"
gbm = "0.15"
drm = "0.11"
wgpu = "0.20"
tracing = "0.1"
anyhow = "1.0"
thiserror = "1.0"

[features]
default = ["dmabuf-support"]
dmabuf-support = ["gbm", "drm"]
```

## 🏗️ Core Implementation

### **1. Wayland Client Foundation**

Create `src/wayland_client.rs`:

```rust
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_manager_v1::{self, ZwlrScreencopyManagerV1},
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
};
use wayland_client::protocol::{
    wl_output::{self, WlOutput},
    wl_registry::{self, WlRegistry},
    wl_shm::{self, WlShm},
};

#[derive(Debug)]
pub struct NiriScreencopyClient {
    connection: Connection,
    screencopy_manager: Option<ZwlrScreencopyManagerV1>,
    outputs: HashMap<u32, WlOutput>,
    shm: Option<WlShm>,
    active_captures: HashMap<u32, ActiveCapture>,
    capture_counter: u32,
}

#[derive(Debug)]
struct ActiveCapture {
    frame: ZwlrScreencopyFrameV1,
    buffer: CaptureBuffer,
    start_time: std::time::Instant,
    completed: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub enum CaptureBuffer {
    Dmabuf {
        dmabuf: gbm::BufferObject,
        width: u32,
        height: u32,
        format: u32,
    },
    Shm {
        data: Vec<u8>,
        width: u32,
        height: u32,
        stride: u32,
    },
}

impl NiriScreencopyClient {
    pub fn new() -> Result<Self> {
        let connection = Connection::connect_to_env()?;
        
        Ok(Self {
            connection,
            screencopy_manager: None,
            outputs: HashMap::new(),
            shm: None,
            active_captures: HashMap::new(),
            capture_counter: 0,
        })
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        let mut event_queue = self.connection.new_event_queue();
        let qh = event_queue.handle();
        
        // Get registry and bind globals
        let display = self.connection.display();
        let registry = display.get_registry(&qh, ());
        
        // Roundtrip to get all globals
        event_queue.roundtrip(self)?;
        
        Ok(())
    }
    
    pub fn capture_output(&mut self, output: &WlOutput, prefer_dmabuf: bool) -> Result<u32> {
        let capture_id = self.capture_counter;
        self.capture_counter += 1;
        
        let screencopy_manager = self.screencopy_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Screencopy manager not available"))?;
        
        // Create screencopy frame
        let mut event_queue = self.connection.new_event_queue();
        let qh = event_queue.handle();
        
        let frame = screencopy_manager.capture_output(
            false, // overlay_cursor
            output,
            &qh,
            capture_id,
        );
        
        // Allocate buffer based on preference
        let buffer = if prefer_dmabuf {
            self.allocate_dmabuf_buffer(1920, 1080)? // TODO: Get actual output size
        } else {
            self.allocate_shm_buffer(1920, 1080)?
        };
        
        // Copy to buffer
        match &buffer {
            CaptureBuffer::Dmabuf { dmabuf, .. } => {
                // TODO: Convert gbm::BufferObject to wl_buffer
                // frame.copy(&dmabuf_wl_buffer);
            }
            CaptureBuffer::Shm { .. } => {
                // TODO: Create wl_buffer from SHM pool
                // frame.copy(&shm_wl_buffer);
            }
        }
        
        let active_capture = ActiveCapture {
            frame,
            buffer,
            start_time: std::time::Instant::now(),
            completed: Arc::new(Mutex::new(false)),
        };
        
        self.active_captures.insert(capture_id, active_capture);
        
        Ok(capture_id)
    }
    
    fn allocate_dmabuf_buffer(&self, width: u32, height: u32) -> Result<CaptureBuffer> {
        #[cfg(feature = "dmabuf-support")]
        {
            use gbm::{Device, Format};
            use std::fs::OpenOptions;
            
            // Open DRM device
            let drm_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/dri/card0")?;
            
            let device = Device::new(drm_file)?;
            
            // Create buffer object
            let bo = device.create_buffer_object::<()>(
                width,
                height,
                Format::Xrgb8888,
                gbm::BufferObjectFlags::RENDERING,
            )?;
            
            Ok(CaptureBuffer::Dmabuf {
                dmabuf: bo,
                width,
                height,
                format: Format::Xrgb8888 as u32,
            })
        }
        
        #[cfg(not(feature = "dmabuf-support"))]
        {
            Err(anyhow::anyhow!("DMA-BUF support not compiled"))
        }
    }
    
    fn allocate_shm_buffer(&self, width: u32, height: u32) -> Result<CaptureBuffer> {
        let stride = width * 4; // RGBA
        let size = (stride * height) as usize;
        let data = vec![0u8; size];
        
        Ok(CaptureBuffer::Shm {
            data,
            width,
            height,
            stride,
        })
    }
    
    pub fn process_events(&mut self) -> Result<Vec<CompletedCapture>> {
        let mut event_queue = self.connection.new_event_queue();
        
        // Process all pending events
        event_queue.roundtrip(self)?;
        
        // Check for completed captures
        let mut completed = Vec::new();
        self.active_captures.retain(|&capture_id, active_capture| {
            if *active_capture.completed.lock().unwrap() {
                completed.push(CompletedCapture {
                    capture_id,
                    buffer: active_capture.buffer.clone(),
                    duration: active_capture.start_time.elapsed(),
                });
                false // Remove from active captures
            } else {
                true // Keep in active captures
            }
        });
        
        Ok(completed)
    }
}

#[derive(Debug)]
pub struct CompletedCapture {
    pub capture_id: u32,
    pub buffer: CaptureBuffer,
    pub duration: std::time::Duration,
}
```

### **2. Bevy Integration Plugin**

Create `src/bevy_plugin.rs`:

```rust
use bevy::prelude::*;
use bevy::render::texture::{Image, TextureFormat};
use bevy::asset::{Assets, Handle};
use std::sync::{Arc, Mutex};
use crate::wayland_client::{NiriScreencopyClient, CaptureBuffer, CompletedCapture};

pub struct NiriCapturePlugin {
    pub target_output: Option<String>,
    pub prefer_dmabuf: bool,
    pub capture_fps: f32,
}

impl Default for NiriCapturePlugin {
    fn default() -> Self {
        Self {
            target_output: None,
            prefer_dmabuf: true,
            capture_fps: 60.0,
        }
    }
}

impl Plugin for NiriCapturePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NiriCaptureConfig {
                target_output: self.target_output.clone(),
                prefer_dmabuf: self.prefer_dmabuf,
                capture_fps: self.capture_fps,
            })
            .add_systems(Startup, setup_niri_capture)
            .add_systems(Update, (
                niri_capture_system,
                update_niri_textures,
            ).chain());
    }
}

#[derive(Resource)]
struct NiriCaptureConfig {
    target_output: Option<String>,
    prefer_dmabuf: bool,
    capture_fps: f32,
}

#[derive(Resource)]
struct NiriCaptureState {
    client: Arc<Mutex<NiriScreencopyClient>>,
    converter: HybridBevyConverter,
    last_capture_time: std::time::Instant,
    current_texture: Option<Handle<Image>>,
    performance_stats: PerformanceStats,
}

#[derive(Component)]
pub struct NiriScreenDisplay {
    pub output_name: Option<String>,
    pub auto_resize: bool,
    pub damage_tracking: bool,
}

fn setup_niri_capture(
    mut commands: Commands,
    config: Res<NiriCaptureConfig>,
) {
    let mut client = NiriScreencopyClient::new()
        .expect("Failed to create Niri screencopy client");
    
    client.initialize()
        .expect("Failed to initialize Wayland connection");
    
    let converter = HybridBevyConverter::new()
        .expect("Failed to create texture converter");
    
    commands.insert_resource(NiriCaptureState {
        client: Arc::new(Mutex::new(client)),
        converter,
        last_capture_time: std::time::Instant::now(),
        current_texture: None,
        performance_stats: PerformanceStats::default(),
    });
}

fn niri_capture_system(
    mut capture_state: ResMut<NiriCaptureState>,
    config: Res<NiriCaptureConfig>,
) {
    let now = std::time::Instant::now();
    let frame_duration = std::time::Duration::from_secs_f32(1.0 / config.capture_fps);
    
    // Check if it's time for next capture
    if now.duration_since(capture_state.last_capture_time) >= frame_duration {
        let mut client = capture_state.client.lock().unwrap();
        
        // Process completed captures first
        if let Ok(completed_captures) = client.process_events() {
            for capture in completed_captures {
                capture_state.performance_stats.record_capture(&capture);
            }
        }
        
        // Request new capture
        if let Some(output) = client.outputs.values().next() {
            let prefer_dmabuf = capture_state.performance_stats
                .should_prefer_dmabuf()
                .unwrap_or(config.prefer_dmabuf);
            
            if let Ok(_capture_id) = client.capture_output(output, prefer_dmabuf) {
                capture_state.last_capture_time = now;
            }
        }
    }
}

fn update_niri_textures(
    mut capture_state: ResMut<NiriCaptureState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, With<NiriScreenDisplay>>,
) {
    let mut client = capture_state.client.lock().unwrap();
    
    // Process completed captures
    if let Ok(completed_captures) = client.process_events() {
        for capture in completed_captures {
            // Convert buffer to Bevy texture
            if let Ok(texture_handle) = capture_state.converter
                .convert_to_bevy_texture(capture.buffer, &mut images) {
                
                // Update all materials using Niri screen display
                for material_handle in query.iter() {
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color_texture = Some(texture_handle.clone());
                    }
                }
                
                capture_state.current_texture = Some(texture_handle);
            }
        }
    }
}

struct HybridBevyConverter {
    // GPU device for DMA buffer conversion
    #[cfg(feature = "dmabuf-support")]
    device: wgpu::Device,
    #[cfg(feature = "dmabuf-support")]
    queue: wgpu::Queue,
}

impl HybridBevyConverter {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "dmabuf-support")]
        {
            // Initialize WGPU for DMA buffer conversion
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })).ok_or("Failed to find suitable adapter")?;
            
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Niri Capture Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            ))?;
            
            Ok(Self { device, queue })
        }
        
        #[cfg(not(feature = "dmabuf-support"))]
        {
            Ok(Self {})
        }
    }
    
    fn convert_to_bevy_texture(
        &mut self,
        buffer: CaptureBuffer,
        images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, Box<dyn std::error::Error>> {
        match buffer {
            CaptureBuffer::Dmabuf { dmabuf, width, height, .. } => {
                self.convert_dmabuf_to_texture(dmabuf, width, height, images)
            }
            CaptureBuffer::Shm { data, width, height, stride } => {
                self.convert_shm_to_texture(data, width, height, stride, images)
            }
        }
    }
    
    #[cfg(feature = "dmabuf-support")]
    fn convert_dmabuf_to_texture(
        &self,
        dmabuf: gbm::BufferObject,
        width: u32,
        height: u32,
        images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, Box<dyn std::error::Error>> {
        // Zero-copy GPU texture import
        // This is a simplified version - actual implementation would need
        // proper DMA-BUF to WGPU texture conversion
        
        // For now, fall back to reading the buffer and creating a regular texture
        // TODO: Implement true zero-copy DMA-BUF import
        let data = vec![0u8; (width * height * 4) as usize]; // Placeholder
        
        let image = Image::new(
            bevy::render::texture::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            bevy::render::texture::TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        Ok(images.add(image))
    }
    
    #[cfg(not(feature = "dmabuf-support"))]
    fn convert_dmabuf_to_texture(
        &self,
        _dmabuf: gbm::BufferObject,
        _width: u32,
        _height: u32,
        _images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, Box<dyn std::error::Error>> {
        Err("DMA-BUF support not compiled".into())
    }
    
    fn convert_shm_to_texture(
        &self,
        data: Vec<u8>,
        width: u32,
        height: u32,
        _stride: u32,
        images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, Box<dyn std::error::Error>> {
        let image = Image::new(
            bevy::render::texture::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            bevy::render::texture::TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        Ok(images.add(image))
    }
}

#[derive(Default)]
struct PerformanceStats {
    dmabuf_captures: u32,
    dmabuf_total_time: std::time::Duration,
    shm_captures: u32,
    shm_total_time: std::time::Duration,
}

impl PerformanceStats {
    fn record_capture(&mut self, capture: &CompletedCapture) {
        match &capture.buffer {
            CaptureBuffer::Dmabuf { .. } => {
                self.dmabuf_captures += 1;
                self.dmabuf_total_time += capture.duration;
            }
            CaptureBuffer::Shm { .. } => {
                self.shm_captures += 1;
                self.shm_total_time += capture.duration;
            }
        }
    }
    
    fn should_prefer_dmabuf(&self) -> Option<bool> {
        if self.dmabuf_captures > 10 && self.shm_captures > 10 {
            let dmabuf_avg = self.dmabuf_total_time / self.dmabuf_captures;
            let shm_avg = self.shm_total_time / self.shm_captures;
            Some(dmabuf_avg < shm_avg)
        } else {
            None
        }
    }
}
```

## 🧪 Testing & Debugging

### **Unit Tests**

Create `tests/integration_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wayland_client_creation() {
        // Test basic client creation
        let client = NiriScreencopyClient::new();
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_buffer_allocation() {
        let client = NiriScreencopyClient::new().unwrap();
        
        // Test SHM buffer allocation
        let shm_buffer = client.allocate_shm_buffer(1920, 1080);
        assert!(shm_buffer.is_ok());
        
        // Test DMA buffer allocation (if supported)
        #[cfg(feature = "dmabuf-support")]
        {
            let dmabuf_buffer = client.allocate_dmabuf_buffer(1920, 1080);
            // May fail on systems without DMA-BUF support
        }
    }
    
    #[test]
    fn test_performance_stats() {
        let mut stats = PerformanceStats::default();
        
        // Simulate some captures
        for _ in 0..20 {
            let capture = CompletedCapture {
                capture_id: 0,
                buffer: CaptureBuffer::Shm {
                    data: vec![],
                    width: 1920,
                    height: 1080,
                    stride: 7680,
                },
                duration: std::time::Duration::from_millis(10),
            };
            stats.record_capture(&capture);
        }
        
        assert_eq!(stats.shm_captures, 20);
    }
}
```

## 📚 Examples

### **Example 1: Basic Screen Mirror**

```rust
// examples/basic_mirror.rs
use bevy::prelude::*;
use bevy_niri_integration::{NiriCapturePlugin, NiriScreenDisplay};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NiriCapturePlugin::default())
        .add_systems(Startup, setup_mirror)
        .run();
}

fn setup_mirror(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a simple mirror of the Niri screen
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(8.0, 6.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true, // No lighting for screen content
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        NiriScreenDisplay {
            output_name: None,
            auto_resize: true,
            damage_tracking: true,
        },
    ));
    
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
```

## 🚀 Future Enhancements

### **Planned Features**

1. **HDR Support**
   - HDR10 and Dolby Vision capture
   - Wide color gamut handling
   - Tone mapping for SDR displays

2. **Advanced Damage Tracking**
   - Region-based updates
   - Predictive damage tracking
   - Temporal coherence optimization

3. **Multi-GPU Support**
   - Cross-GPU buffer sharing
   - Load balancing across GPUs
   - Fallback GPU selection

4. **Compression & Streaming**
   - Real-time compression
   - Network streaming support
   - Adaptive bitrate control

---

**Document Version:** 1.0  
**Last Updated:** July 30, 2025  
**Authors:** Devin AI for walue-dev  
**Status:** Development Guide - Ready for Implementation
