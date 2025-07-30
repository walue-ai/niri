# Bevy-Niri Integration - Technical Architecture

**Project**: Multi-Screen Real-time Compositor Integration  
**Version**: 1.0.0  
**Target Performance**: 60+ FPS, <2ms latency  
**Last Updated**: July 30, 2025

## 🏗 System Architecture Overview

The Bevy-Niri integration creates a high-performance bridge between the Niri Wayland compositor and Bevy game engine, enabling real-time display of multiple screen outputs within a single Bevy application.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Bevy Application Layer                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │   Multi-Screen  │  │   Performance   │  │  Configuration  │             │
│  │    Display      │  │   Monitoring    │  │    Management   │             │
│  │    System       │  │     System      │  │     System      │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Bevy-Niri Integration Layer                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │  NiriCapture    │  │   Adaptive      │  │    Texture      │             │
│  │    Plugin       │  │  Performance    │  │   Pipeline      │             │
│  │                 │  │   Manager       │  │                 │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
├─────────────────────────────────────────────────────────────────────────────┤
│                         Buffer Management Layer                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │   DMA Buffer    │  │   SHM Buffer    │  │    Buffer       │             │
│  │    Manager      │  │    Manager      │  │   Allocator     │             │
│  │                 │  │                 │  │                 │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
├─────────────────────────────────────────────────────────────────────────────┤
│                        Wayland Protocol Layer                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │   Screencopy    │  │     Output      │  │     Event       │             │
│  │    Client       │  │   Management    │  │    Handler      │             │
│  │                 │  │                 │  │                 │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
├─────────────────────────────────────────────────────────────────────────────┤
│                           Hardware Layer                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │      GPU        │  │   Wayland       │  │      Niri       │             │
│  │   (Vulkan/GL)   │  │  Compositor     │  │   Compositor    │             │
│  │                 │  │                 │  │                 │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🔧 Core Components

### 1. NiriCapturePlugin (Bevy Integration)

The main Bevy plugin that orchestrates the entire integration.

```rust
pub struct NiriCapturePlugin {
    pub outputs: Vec<String>,           // Target output names
    pub capture_fps: f32,               // Target frame rate
    pub prefer_dmabuf: bool,            // Prefer DMA buffers
    pub fallback_to_shm: bool,          // Allow SHM fallback
    pub adaptive_performance: bool,      // Enable adaptive optimization
}

impl Plugin for NiriCapturePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NiriCaptureState::new(self.clone()))
            .add_systems(Startup, initialize_wayland_client)
            .add_systems(Update, (
                capture_screens_system,
                update_textures_system,
                monitor_performance_system,
                adapt_strategy_system,
            ).chain())
            .add_systems(PostUpdate, cleanup_resources_system);
    }
}
```

**Responsibilities**:
- Initialize Wayland connection
- Manage capture lifecycle
- Coordinate with Bevy's rendering pipeline
- Handle resource cleanup

### 2. Wayland Screencopy Client

Low-level Wayland protocol implementation for screen capture.

```rust
pub struct NiriScreencopyClient {
    connection: Connection,
    event_queue: EventQueue<AppData>,
    screencopy_manager: ZwlrScreencopyManagerV1,
    outputs: HashMap<String, OutputInfo>,
    active_captures: HashMap<String, CaptureState>,
}

impl NiriScreencopyClient {
    pub fn capture_output(&mut self, output_name: &str) -> Result<CaptureBuffer, CaptureError> {
        // 1. Get output handle
        let output = self.outputs.get(output_name)?;
        
        // 2. Create screencopy frame
        let frame = self.screencopy_manager.capture_output(0, &output.handle, &self.event_queue.handle(), ());
        
        // 3. Allocate buffer (DMA or SHM)
        let buffer = self.allocate_buffer(&output.info)?;
        
        // 4. Attach buffer to frame
        frame.copy(&buffer.wl_buffer);
        
        // 5. Wait for completion
        self.wait_for_capture_complete()?;
        
        Ok(buffer)
    }
}
```

**Key Features**:
- Asynchronous capture operations
- Multiple output support
- Buffer format negotiation
- Error handling and recovery

### 3. Buffer Management System

Efficient memory management for both DMA and SHM buffers.

#### DMA Buffer Manager
```rust
pub struct DmaBufferManager {
    gbm_device: GbmDevice<DrmDevice>,
    allocator: GbmAllocator,
    buffer_pool: Vec<DmaBuffer>,
    vulkan_device: Device,
}

impl DmaBufferManager {
    pub fn allocate_buffer(&mut self, width: u32, height: u32) -> Result<DmaBuffer, DmaError> {
        // 1. Check buffer pool for reusable buffer
        if let Some(buffer) = self.find_reusable_buffer(width, height) {
            return Ok(buffer);
        }
        
        // 2. Create new GBM buffer object
        let bo = self.gbm_device.create_buffer_object::<()>(
            width, height,
            Format::Argb8888,
            BufferObjectFlags::RENDERING | BufferObjectFlags::SCANOUT,
        )?;
        
        // 3. Export as DMA-BUF
        let fd = bo.fd()?;
        let stride = bo.stride()?;
        let offset = bo.offset(0)?;
        
        // 4. Import into Vulkan
        let vulkan_image = self.import_to_vulkan(fd, width, height)?;
        
        // 5. Create Bevy texture handle
        let texture_handle = self.create_bevy_texture(vulkan_image)?;
        
        Ok(DmaBuffer {
            bo,
            fd,
            stride,
            offset,
            vulkan_image,
            texture_handle,
            width,
            height,
        })
    }
}
```

#### SHM Buffer Manager
```rust
pub struct ShmBufferManager {
    shm: WlShm,
    buffer_pool: Vec<ShmBuffer>,
    memory_pool: MemoryPool,
}

impl ShmBufferManager {
    pub fn allocate_buffer(&mut self, width: u32, height: u32) -> Result<ShmBuffer, ShmError> {
        let stride = width * 4; // RGBA8888
        let size = stride * height;
        
        // 1. Allocate shared memory
        let memory = self.memory_pool.allocate(size as usize)?;
        
        // 2. Create Wayland buffer
        let wl_buffer = self.shm.create_buffer(
            memory.fd(),
            width as i32,
            height as i32,
            stride as i32,
            WlShmFormat::Argb8888,
        );
        
        // 3. Create Bevy texture
        let texture_data = unsafe { 
            std::slice::from_raw_parts_mut(memory.as_mut_ptr(), size as usize) 
        };
        let texture_handle = self.create_bevy_texture_from_data(texture_data, width, height)?;
        
        Ok(ShmBuffer {
            memory,
            wl_buffer,
            texture_handle,
            width,
            height,
            stride,
        })
    }
}
```

### 4. Adaptive Performance Manager

Intelligent system for optimizing performance based on real-time metrics.

```rust
pub struct AdaptivePerformanceManager {
    metrics: PerformanceMetrics,
    strategy: CaptureStrategy,
    history: VecDeque<PerformanceMetrics>,
    thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub frame_rate: f32,
    pub latency_ms: f32,
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub memory_usage: usize,
    pub dropped_frames: u32,
    pub buffer_allocation_time: f32,
    pub texture_upload_time: f32,
}

impl AdaptivePerformanceManager {
    pub fn update_strategy(&mut self, metrics: PerformanceMetrics) -> CaptureStrategy {
        self.history.push_back(metrics.clone());
        self.metrics = metrics;
        
        // Calculate performance score
        let performance_score = self.calculate_performance_score();
        
        // Determine optimal strategy
        match performance_score {
            score if score > 0.8 => {
                // Excellent performance - use DMA with high quality
                CaptureStrategy::DmaHighQuality
            }
            score if score > 0.6 => {
                // Good performance - use DMA with standard quality
                CaptureStrategy::DmaStandard
            }
            score if score > 0.4 => {
                // Moderate performance - use SHM with optimization
                CaptureStrategy::ShmOptimized
            }
            _ => {
                // Poor performance - use basic SHM
                CaptureStrategy::ShmBasic
            }
        }
    }
    
    fn calculate_performance_score(&self) -> f32 {
        let frame_rate_score = (self.metrics.frame_rate / 60.0).min(1.0);
        let latency_score = (10.0 / self.metrics.latency_ms).min(1.0);
        let cpu_score = (1.0 - self.metrics.cpu_usage / 100.0).max(0.0);
        let memory_score = if self.metrics.memory_usage < 100_000_000 { 1.0 } else { 0.5 };
        
        (frame_rate_score * 0.4 + latency_score * 0.3 + cpu_score * 0.2 + memory_score * 0.1)
    }
}

#[derive(Debug, Clone)]
pub enum CaptureStrategy {
    DmaHighQuality,     // 60+ FPS, <2ms latency
    DmaStandard,        // 45+ FPS, <5ms latency
    ShmOptimized,       // 30+ FPS, <10ms latency
    ShmBasic,           // 15+ FPS, <20ms latency
}
```

### 5. Multi-Screen Display System

Manages multiple screen outputs within a single Bevy scene.

```rust
pub struct MultiScreenDisplaySystem {
    screens: HashMap<String, ScreenDisplay>,
    layout_manager: ScreenLayoutManager,
    interaction_handler: ScreenInteractionHandler,
}

#[derive(Component)]
pub struct ScreenDisplay {
    pub output_name: String,
    pub texture_handle: Handle<Image>,
    pub transform: Transform,
    pub scale: Vec3,
    pub interactive: bool,
}

impl MultiScreenDisplaySystem {
    pub fn setup_screens(&mut self, 
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        outputs: &[OutputInfo]
    ) {
        for (index, output) in outputs.iter().enumerate() {
            let position = self.layout_manager.calculate_position(index, outputs.len());
            let scale = self.layout_manager.calculate_scale(&output);
            
            let screen_entity = commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Plane3d::new(Vec3::Z, Vec2::new(output.width as f32 / 1000.0, output.height as f32 / 1000.0)).mesh()),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(output.texture_handle.clone()),
                        unlit: true,
                        ..default()
                    }),
                    transform: Transform::from_translation(position).with_scale(scale),
                    ..default()
                },
                ScreenDisplay {
                    output_name: output.name.clone(),
                    texture_handle: output.texture_handle.clone(),
                    transform: Transform::from_translation(position).with_scale(scale),
                    scale,
                    interactive: true,
                },
                Name::new(format!("Screen-{}", output.name)),
            )).id();
            
            self.screens.insert(output.name.clone(), screen_entity);
        }
    }
}
```

## 🔄 Data Flow Architecture

### Capture Pipeline
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    Niri     │    │  Wayland    │    │   Buffer    │    │    Bevy     │
│ Compositor  │───▶│ Screencopy  │───▶│ Management  │───▶│   Texture   │
│             │    │  Protocol   │    │             │    │  Pipeline   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Output    │    │   Frame     │    │ DMA/SHM     │    │   Render    │
│  Changes    │    │  Request    │    │  Buffer     │    │   Target    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### Performance Monitoring Loop
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Metrics    │    │  Analysis   │    │  Strategy   │
│ Collection  │───▶│   Engine    │───▶│ Adaptation  │
│             │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                                       │
       │                                       ▼
┌─────────────┐                      ┌─────────────┐
│ Performance │                      │ Buffer Type │
│  Feedback   │◄─────────────────────│  Selection  │
└─────────────┘                      └─────────────┘
```

## 🏛 System Interfaces

### 1. Wayland Protocol Interface

```rust
pub trait WaylandScreencopyInterface {
    fn connect_to_compositor(&mut self) -> Result<(), WaylandError>;
    fn enumerate_outputs(&mut self) -> Result<Vec<OutputInfo>, WaylandError>;
    fn capture_output(&mut self, output_name: &str, buffer_type: BufferType) -> Result<CaptureBuffer, CaptureError>;
    fn poll_events(&mut self) -> Result<Vec<WaylandEvent>, WaylandError>;
    fn cleanup(&mut self) -> Result<(), WaylandError>;
}
```

### 2. Buffer Management Interface

```rust
pub trait BufferManager {
    type Buffer: CaptureBuffer;
    type Error: std::error::Error;
    
    fn allocate_buffer(&mut self, width: u32, height: u32) -> Result<Self::Buffer, Self::Error>;
    fn deallocate_buffer(&mut self, buffer: Self::Buffer) -> Result<(), Self::Error>;
    fn get_buffer_info(&self, buffer: &Self::Buffer) -> BufferInfo;
    fn is_buffer_ready(&self, buffer: &Self::Buffer) -> bool;
}
```

### 3. Bevy Integration Interface

```rust
pub trait BevyTextureProvider {
    fn create_texture_from_buffer(&mut self, buffer: &CaptureBuffer) -> Result<Handle<Image>, TextureError>;
    fn update_texture(&mut self, handle: &Handle<Image>, buffer: &CaptureBuffer) -> Result<(), TextureError>;
    fn get_texture_info(&self, handle: &Handle<Image>) -> Option<TextureInfo>;
}
```

## 🔧 Configuration Architecture

### Hybrid Configuration System

```rust
// KDL Configuration (config.kdl)
bevy-niri {
    outputs ["DP-1", "HDMI-A-1"]
    capture-fps 60.0
    prefer-dmabuf true
    adaptive-performance true
    
    performance {
        target-fps 60.0
        max-latency-ms 2.0
        memory-limit-mb 100
    }
    
    display {
        layout "grid"
        spacing 0.1
        scale-factor 1.0
    }
}
```

```rust
// Rust API Configuration
let config = NiriCaptureConfig {
    outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
    capture_fps: 60.0,
    prefer_dmabuf: true,
    adaptive_performance: true,
    performance: PerformanceConfig {
        target_fps: 60.0,
        max_latency_ms: 2.0,
        memory_limit_mb: 100,
    },
    display: DisplayConfig {
        layout: LayoutType::Grid,
        spacing: 0.1,
        scale_factor: 1.0,
    },
};

let plugin = NiriCapturePlugin::from_config(config);
```

## 📊 Performance Architecture

### Target Performance Metrics

| Component | Target | Measurement Method |
|-----------|--------|-------------------|
| **Frame Rate** | 60+ FPS | Frame time measurement |
| **Latency (DMA)** | <2ms | Capture-to-display timing |
| **Latency (SHM)** | <10ms | Capture-to-display timing |
| **CPU Usage** | <5% | System resource monitoring |
| **Memory Usage** | <100MB | Heap allocation tracking |
| **GPU Usage** | <30% | GPU performance counters |

### Performance Optimization Strategies

1. **Zero-Copy Transfers**: DMA buffers eliminate CPU-GPU memory copies
2. **Buffer Pooling**: Reuse allocated buffers to reduce allocation overhead
3. **Parallel Processing**: Capture and rendering in separate threads
4. **Adaptive Quality**: Dynamic quality adjustment based on performance
5. **Damage Tracking**: Only update changed screen regions

## 🔒 Error Handling Architecture

### Error Classification System

```rust
#[derive(Debug, thiserror::Error)]
pub enum NiriCaptureError {
    #[error("Wayland connection failed: {0}")]
    WaylandConnection(String),
    
    #[error("Buffer allocation failed: {0}")]
    BufferAllocation(String),
    
    #[error("Texture creation failed: {0}")]
    TextureCreation(String),
    
    #[error("Performance degradation: {0}")]
    PerformanceDegradation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
```

### Recovery Strategies

1. **Automatic Fallback**: DMA → SHM → Software rendering
2. **Graceful Degradation**: Reduce quality/framerate to maintain stability
3. **Resource Cleanup**: Automatic cleanup on errors
4. **User Notification**: Clear error messages with suggested actions

## 🧪 Testing Architecture

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Cross-component interaction testing
3. **Performance Tests**: Benchmark and regression testing
4. **Compatibility Tests**: Multi-platform and hardware testing

### Test Infrastructure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_capture_performance(c: &mut Criterion) {
        c.bench_function("dma_buffer_capture", |b| {
            b.iter(|| {
                // Benchmark DMA buffer capture
                black_box(capture_with_dma_buffer())
            })
        });
        
        c.bench_function("shm_buffer_capture", |b| {
            b.iter(|| {
                // Benchmark SHM buffer capture
                black_box(capture_with_shm_buffer())
            })
        });
    }
    
    criterion_group!(benches, benchmark_capture_performance);
    criterion_main!(benches);
}
```

## 🚀 Deployment Architecture

### Build Configuration

```toml
[package]
name = "bevy-niri-integration"
version = "1.0.0"
edition = "2021"

[dependencies]
bevy = { version = "0.14", features = ["wayland"] }
wayland-client = "0.31"
wayland-protocols-wlr = "0.2"
smithay-client-toolkit = "0.18"
gbm = "0.15"
drm = "0.11"
wgpu = "0.20"

[features]
default = ["dma-buffers", "adaptive-performance"]
dma-buffers = ["gbm", "drm"]
adaptive-performance = ["metrics"]
debug-mode = ["tracing-subscriber"]
```

### Runtime Requirements

- **OS**: Linux with Wayland support
- **Compositor**: Niri with screencopy_v1 protocol
- **GPU**: Vulkan 1.1+ or OpenGL 3.3+ support
- **Memory**: 4GB+ RAM recommended
- **CPU**: Multi-core processor for parallel processing

## 🔮 Future Architecture Considerations

### Planned Extensions

1. **Multi-GPU Support**: Distribute capture across multiple GPUs
2. **Network Streaming**: Remote screen capture over network
3. **Recording Integration**: Built-in screen recording capabilities
4. **Plugin System**: Extensible plugin architecture for custom processing
5. **VR/AR Integration**: Support for immersive display environments

### Scalability Considerations

1. **Horizontal Scaling**: Support for distributed capture systems
2. **Vertical Scaling**: Optimize for high-resolution displays (4K+, 8K)
3. **Resource Management**: Dynamic resource allocation based on load
4. **Performance Monitoring**: Real-time performance analytics and alerting

---

**Link to Devin run**: https://app.devin.ai/sessions/d0f3ea092883490e904ec5a21c673b9c  
**Requested by**: @walue-dev  
**Architecture Version**: 1.0.0  
**Last Updated**: July 30, 2025 14:50 UTC
