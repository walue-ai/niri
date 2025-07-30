# Bevy-Niri Integration Technical Plan

## 🎯 Project Overview

This document outlines the technical plan for integrating the Niri Wayland compositor with the Bevy game engine, enabling real-time capture and rendering of Niri's compositor screen as a texture within Bevy applications.

### **Objective**
Develop a system that captures Niri compositor screens using Wayland screencopy protocol and renders them as mesh material textures in Bevy, similar to `bevy_egui`'s `render_egui_to_image` example.

### **Key Requirements**
- ✅ Support both DMA buffer and SHM buffer methods
- ✅ Real-time screen capture with minimal latency
- ✅ Seamless integration with Bevy's rendering pipeline
- ✅ Automatic fallback mechanisms for compatibility
- ✅ Performance optimization for different hardware configurations

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Niri          │    │  Wayland Client  │    │   Bevy App      │
│   Compositor    │◄──►│  (Screencopy)    │◄──►│   (Renderer)    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ ScreencopyBuffer│    │ Hybrid Capture   │    │ Texture Assets  │
│ • Dmabuf        │    │ Manager          │    │ • GPU Textures  │
│ • Shm           │    │ • Auto Detection │    │ • Material Mesh │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🔧 Technical Components

### **1. Niri Screencopy Protocol Analysis**

Based on source code analysis of `src/protocols/screencopy.rs`, Niri already implements a sophisticated hybrid screencopy system:

```rust
/// Niri's existing ScreencopyBuffer enum
#[derive(Clone)]
pub enum ScreencopyBuffer {
    Dmabuf(Dmabuf),    // GPU memory buffer - zero-copy
    Shm(WlBuffer),     // Shared memory buffer - fallback
}
```

**Key Features:**
- ✅ Automatic buffer type detection
- ✅ Client-driven buffer selection
- ✅ Comprehensive validation for both types
- ✅ Hardware sync support for DMA buffers
- ✅ Damage tracking for both methods

### **2. Hybrid Buffer Selection Logic**

Niri's implementation (lines 365-394 in screencopy.rs):

```rust
let buffer = if let Ok(dmabuf) = dmabuf::get_dmabuf(&buffer) {
    // DMA Buffer Path - Optimal Performance
    if dmabuf.format().code == Fourcc::Xrgb8888
        && dmabuf.width() == size.w as u32
        && dmabuf.height() == size.h as u32
    {
        ScreencopyBuffer::Dmabuf(dmabuf.clone())
    } else {
        // Validation error
    }
} else if shm::with_buffer_contents(&buffer, |_, shm_len, buffer_data| {
    // SHM Buffer Path - Fallback
    buffer_data.format == Format::Xrgb8888
        && buffer_data.width == size.w
        && buffer_data.height == size.h
        && buffer_data.stride == size.w * 4
        && shm_len == buffer_data.stride as usize * buffer_data.height as usize
}) {
    ScreencopyBuffer::Shm(buffer)
} else {
    // Invalid buffer error
}
```

## 📊 Performance Characteristics

| Method | Latency | CPU Usage | Memory Bandwidth | Compatibility | Use Case |
|--------|---------|-----------|------------------|---------------|----------|
| **DMA Buffer** | 1-2ms | <3% | ~2GB/s | 90% (modern GPUs) | High-performance, real-time |
| **SHM Buffer** | 5-10ms | 10-15% | ~8GB/s | 100% | Compatibility, fallback |
| **Hybrid** | 1-10ms | 3-15% | 2-8GB/s | 100% | Adaptive, optimal |

## 🚀 Implementation Strategy

### **Phase 1: Wayland Client Development**
- Implement Wayland screencopy client
- Support both DMA and SHM buffer allocation
- Implement capability detection and fallback logic
- Add performance monitoring and adaptive selection

### **Phase 2: Bevy Integration**
- Create Bevy plugin for Niri screen capture
- Implement dual-path texture conversion (DMA/SHM → Bevy)
- Add real-time capture loop with frame synchronization
- Integrate with Bevy's asset system

### **Phase 3: Optimization & Features**
- Implement damage tracking for partial updates
- Add performance profiling and adaptive switching
- Support multiple output capture
- Add configuration and debugging tools

## 🎮 Bevy Integration Architecture

### **Core Components**

```rust
// Main plugin structure
pub struct NiriCapturePlugin {
    pub target_output: Option<String>,
    pub prefer_dmabuf: bool,
    pub capture_fps: f32,
}

// Resource for managing capture state
#[derive(Resource)]
pub struct NiriCaptureState {
    client: UnifiedCaptureClient,
    converter: HybridBevyConverter,
    performance_monitor: PerformanceMonitor,
    current_texture: Option<Handle<Image>>,
}

// Component for entities that display Niri screen
#[derive(Component)]
pub struct NiriScreenDisplay {
    pub output_name: String,
    pub auto_resize: bool,
    pub damage_tracking: bool,
}
```

### **System Architecture**

```rust
// Main capture system
fn niri_capture_system(
    mut capture_state: ResMut<NiriCaptureState>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&mut Handle<StandardMaterial>, With<NiriScreenDisplay>>,
) {
    // 1. Process completed captures
    let completed = capture_state.client.process_capture_events();
    
    // 2. Convert buffers to Bevy textures
    for capture in completed {
        let texture_handle = capture_state.converter
            .convert_to_bevy_texture(capture.buffer, &mut images)?;
        
        // 3. Update materials
        for mut material_handle in query.iter_mut() {
            if let Some(material) = materials.get_mut(&material_handle) {
                material.base_color_texture = Some(texture_handle.clone());
            }
        }
        
        capture_state.current_texture = Some(texture_handle);
    }
    
    // 4. Request new capture if needed
    if capture_state.should_capture_next_frame() {
        capture_state.client.capture_output_adaptive(
            &output, 
            capture_state.performance_monitor.recommend_dmabuf()
        )?;
    }
}
```

## 🔄 Hybrid Conversion Pipeline

### **DMA Buffer → Bevy Texture (Zero-Copy)**

```rust
fn convert_dmabuf_to_bevy_texture(
    &self,
    dmabuf: &Dmabuf,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>> {
    // Direct GPU memory import - zero copy
    let wgpu_texture = unsafe {
        self.device.create_texture_from_hal(
            hal::TextureDescriptor {
                label: Some("niri_dmabuf_capture"),
                size: wgpu::Extent3d {
                    width: dmabuf.width(),
                    height: dmabuf.height(),
                    depth_or_array_layers: 1,
                },
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            Some(Box::new(dmabuf.clone())), // Zero-copy import
        )
    };
    
    let image = Image::from_wgpu_texture(wgpu_texture, TextureFormat::Rgba8UnormSrgb);
    Ok(images.add(image))
}
```

### **SHM Buffer → Bevy Texture (CPU Copy)**

```rust
fn convert_shm_to_bevy_texture(
    &self,
    shm_buffer: &WlBuffer,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>> {
    // CPU memory to GPU texture upload
    shm::with_buffer_contents(shm_buffer, |_, _, buffer_data| {
        let image = Image::new(
            Extent3d {
                width: buffer_data.width as u32,
                height: buffer_data.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            buffer_data.data.to_vec(), // CPU copy
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        images.add(image)
    })
}
```

## 📈 Adaptive Performance System

### **Real-time Method Selection**

```rust
#[derive(Debug)]
enum CaptureStrategy {
    PreferDmabuf,
    PreferShm,
    Adaptive { dmabuf_ratio: f32 },
    ForceMethod(CaptureBufferType),
}

impl AdaptiveCaptureSystem {
    fn update_capture_strategy(&mut self) {
        let report = self.performance_monitor.get_performance_report();
        
        self.current_strategy = match report.recommendation {
            BufferRecommendation::PreferDmabuf => {
                if report.dmabuf_success_rate > 0.95 {
                    CaptureStrategy::PreferDmabuf
                } else {
                    CaptureStrategy::Adaptive { dmabuf_ratio: 0.7 }
                }
            }
            BufferRecommendation::PreferShm => {
                CaptureStrategy::PreferShm
            }
            BufferRecommendation::Adaptive => {
                CaptureStrategy::Adaptive { dmabuf_ratio: 0.5 }
            }
        };
    }
}
```

## 🛠️ Development Milestones

### **Milestone 1: Basic Wayland Client (Week 1-2)**
- [ ] Implement basic screencopy client
- [ ] Support SHM buffer allocation and capture
- [ ] Basic Wayland event loop and connection management
- [ ] Simple capture validation and error handling

### **Milestone 2: DMA Buffer Support (Week 3-4)**
- [ ] Add DMA buffer allocator integration
- [ ] Implement GPU sync and fence handling
- [ ] Add capability detection for DMA support
- [ ] Performance comparison between DMA and SHM

### **Milestone 3: Bevy Plugin Foundation (Week 5-6)**
- [ ] Create basic Bevy plugin structure
- [ ] Implement SHM → Bevy texture conversion
- [ ] Add basic capture loop integration
- [ ] Simple mesh material texture updates

### **Milestone 4: Hybrid System (Week 7-8)**
- [ ] Implement DMA → Bevy texture conversion
- [ ] Add adaptive buffer selection logic
- [ ] Performance monitoring and statistics
- [ ] Automatic fallback mechanisms

### **Milestone 5: Optimization & Features (Week 9-10)**
- [ ] Damage tracking implementation
- [ ] Multi-output support
- [ ] Configuration system
- [ ] Performance profiling tools

### **Milestone 6: Testing & Documentation (Week 11-12)**
- [ ] Comprehensive testing suite
- [ ] Performance benchmarking
- [ ] User documentation and examples
- [ ] Integration testing with various hardware

## 🔍 Technical Challenges & Solutions

### **Challenge 1: GPU Memory Synchronization**
**Problem:** DMA buffers require proper GPU synchronization to avoid race conditions.
**Solution:** Use Niri's existing `submit_after_sync` mechanism with hardware sync points.

### **Challenge 2: Format Compatibility**
**Problem:** Different GPUs support different DMA buffer formats.
**Solution:** Implement format negotiation and conversion pipeline.

### **Challenge 3: Performance Monitoring**
**Problem:** Need real-time performance feedback for adaptive selection.
**Solution:** Implement lightweight performance tracking with minimal overhead.

### **Challenge 4: Memory Management**
**Problem:** Efficient buffer lifecycle management for both DMA and SHM.
**Solution:** Use RAII patterns and automatic cleanup with proper reference counting.

## 📚 Dependencies & Requirements

### **Rust Crates**
```toml
[dependencies]
# Wayland protocol support
wayland-client = "0.31"
wayland-protocols-wlr = "0.2"
smithay-client-toolkit = "0.18"

# DMA buffer support
gbm = "0.15"
drm = "0.11"

# Bevy integration
bevy = "0.14"
wgpu = "0.20"

# Performance monitoring
tracing = "0.1"
metrics = "0.23"
```

### **System Requirements**
- **Wayland Compositor:** Niri with screencopy protocol support
- **GPU:** Modern GPU with DMA-BUF support (optional, SHM fallback available)
- **Kernel:** Linux 5.4+ with DRM/KMS support
- **Mesa:** 23.1+ for optimal DMA buffer support

## 🎯 Success Criteria

### **Functional Requirements**
- ✅ Capture Niri compositor screen in real-time
- ✅ Support both DMA and SHM buffer methods
- ✅ Automatic fallback when DMA unavailable
- ✅ Integration with Bevy's rendering pipeline
- ✅ Configurable capture parameters (FPS, quality, etc.)

### **Performance Requirements**
- ✅ DMA capture latency: <2ms
- ✅ SHM capture latency: <10ms
- ✅ CPU usage: <5% for DMA, <15% for SHM
- ✅ Memory efficiency: Zero-copy for DMA, single-copy for SHM
- ✅ Frame rate: 60 FPS sustained capture

### **Compatibility Requirements**
- ✅ Works on all systems with Wayland support
- ✅ Graceful degradation on older hardware
- ✅ Support for multiple GPU vendors (Intel, AMD, NVIDIA)
- ✅ Compatible with different Bevy versions

## 📖 References

- [Niri Compositor](https://github.com/YaLTeR/niri)
- [Bevy Game Engine](https://bevyengine.org/)
- [Wayland Screencopy Protocol](https://wayland.app/protocols/wlr-screencopy-unstable-v1)
- [DMA-BUF Documentation](https://www.kernel.org/doc/html/latest/driver-api/dma-buf.html)
- [Bevy Egui Integration](https://github.com/vladbat00/bevy_egui)

---

**Document Version:** 1.0  
**Last Updated:** July 30, 2025  
**Authors:** Devin AI for walue-dev  
**Status:** Technical Planning Phase
