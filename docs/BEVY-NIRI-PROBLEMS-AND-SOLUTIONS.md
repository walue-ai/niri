# Bevy-Niri Integration - Problems and Solutions

**Date:** July 30, 2025  
**Status**: Active Development  
**Priority**: Critical Issues Resolution

## 🚨 Critical Problems

### 1. EGL Surface Format Compatibility Issue

#### Problem Description
```
thread 'Compute Task Pool (2)' panicked at bevy_render-0.14.2/src/view/window/mod.rs:476:51:
No supported formats for surface
Encountered a panic in system `bevy_render::view::window::create_surfaces`!
Segmentation fault (core dumped)
```

#### Technical Analysis
- **Location**: `bevy_render::view::window::create_surfaces`
- **Cause**: Vulkan backend cannot find compatible surface formats for Wayland
- **Impact**: Complete application crash, no rendering possible
- **Environment**: NVIDIA GeForce GTX 1050, Ubuntu 22.04, Wayland

#### Root Causes
1. **Wayland-Vulkan Surface Mismatch**: Vulkan surface creation incompatible with Wayland compositor
2. **NVIDIA Driver Limitations**: Driver may not expose required formats for Wayland surfaces
3. **Bevy Surface Format Detection**: Bevy's format enumeration failing on this GPU/driver combination

#### Solution Approaches

##### Approach 1: Software Rendering Fallback (Immediate)
```rust
// Force OpenGL backend instead of Vulkan
export WGPU_BACKEND=gl
export MESA_GL_VERSION_OVERRIDE=3.3

// In Bevy app configuration
.add_plugins(DefaultPlugins.set(RenderPlugin {
    render_creation: RenderCreation::Automatic(WgpuSettings {
        backends: Some(Backends::GL),
        ..default()
    }),
    ..default()
}))
```

**Pros**: 
- Immediate workaround
- Bypasses Vulkan issues
- Compatible with most systems

**Cons**: 
- Reduced performance
- May not achieve 60+ FPS target
- Limited GPU acceleration

##### Approach 2: Vulkan Surface Format Override (Medium-term)
```rust
// Custom surface format selection
impl NiriCapturePlugin {
    fn create_compatible_surface(&self, instance: &Instance, window: &Window) -> Result<Surface, SurfaceError> {
        let surface = unsafe { instance.create_surface(window)? };
        
        // Override format selection with known compatible formats
        let compatible_formats = vec![
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm,
            TextureFormat::Rgba8Unorm,
        ];
        
        // Force format selection
        surface.configure(&SurfaceConfiguration {
            format: compatible_formats[0],
            ..default()
        });
        
        Ok(surface)
    }
}
```

**Pros**: 
- Maintains Vulkan performance
- Targeted fix for format issues
- Preserves GPU acceleration

**Cons**: 
- Requires deep Bevy integration
- May need upstream Bevy changes
- Complex implementation

##### Approach 3: Headless Rendering with Manual Surface (Long-term)
```rust
// Bypass window surface creation entirely
pub struct HeadlessNiriCapture {
    device: Device,
    queue: Queue,
    texture: Texture,
}

impl HeadlessNiriCapture {
    pub fn new() -> Self {
        // Create headless Vulkan context
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = instance.request_adapter(&RequestAdapterOptions::default()).await;
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default(), None).await;
        
        // Create offscreen texture for rendering
        let texture = device.create_texture(&TextureDescriptor {
            size: Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            ..default()
        });
        
        Self { device, queue, texture }
    }
}
```

**Pros**: 
- Complete control over rendering
- No window surface dependencies
- Maximum compatibility

**Cons**: 
- Major architectural change
- Complex implementation
- May lose Bevy integration benefits

### 2. Rust Environment Inconsistency

#### Problem Description
SSH sessions not picking up updated Rust 1.88.0, causing build failures:
```
error: package `bevy v0.14.2` cannot be built because it requires rustc 1.79.0 or newer, 
while the currently active rustc version is 1.75.0
```

#### Root Cause
- Rust update applied to user profile but not propagated to SSH sessions
- PATH and CARGO_HOME environment variables not updated in SSH context
- Shell profile not sourced automatically in non-interactive SSH

#### Solution
```bash
# Permanent fix for SSH environment
echo 'source ~/.cargo/env' >> ~/.bashrc
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.profile

# Immediate workaround for each SSH session
ssh user@host "source ~/.cargo/env && cd project && cargo build"

# Verify Rust version in SSH context
ssh user@host "source ~/.cargo/env && rustc --version"
```

### 3. DMA Buffer Implementation Gap

#### Problem Description
Current implementation only supports SHM buffers, missing high-performance DMA buffer path required for 60+ FPS target.

#### Missing Components
1. **GBM Device Integration**
2. **DRM Buffer Sharing**
3. **Vulkan External Memory**
4. **Hardware Synchronization**

#### Solution Implementation

##### Phase 1: GBM Integration
```rust
use gbm::{Device as GbmDevice, BufferObject, Format};
use drm::Device as DrmDevice;

pub struct DmaBufferManager {
    gbm_device: GbmDevice<DrmDevice>,
    allocator: GbmAllocator,
}

impl DmaBufferManager {
    pub fn new() -> Result<Self, DmaError> {
        let drm_device = DrmDevice::open("/dev/dri/card0")?;
        let gbm_device = GbmDevice::new(drm_device)?;
        let allocator = GbmAllocator::new(gbm_device.clone());
        
        Ok(Self { gbm_device, allocator })
    }
    
    pub fn create_buffer(&mut self, width: u32, height: u32) -> Result<DmaBuffer, DmaError> {
        let bo = self.gbm_device.create_buffer_object::<()>(
            width, height,
            Format::Argb8888,
            BufferObjectFlags::RENDERING | BufferObjectFlags::SCANOUT,
        )?;
        
        Ok(DmaBuffer::from_buffer_object(bo))
    }
}
```

##### Phase 2: Vulkan External Memory
```rust
use ash::vk;

impl DmaBuffer {
    pub fn import_to_vulkan(&self, device: &Device) -> Result<vk::Image, VulkanError> {
        let external_memory_info = vk::ExternalMemoryImageCreateInfo::builder()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
            
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D { width: self.width, height: self.height, depth: 1 })
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .push_next(&mut external_memory_info);
            
        let image = unsafe { device.create_image(&image_info, None)? };
        
        // Import DMA buffer as Vulkan memory
        let import_info = vk::ImportMemoryFdInfoKHR::builder()
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
            .fd(self.fd);
            
        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(self.find_memory_type_index(&memory_requirements)?)
            .push_next(&mut import_info);
            
        let memory = unsafe { device.allocate_memory(&alloc_info, None)? };
        unsafe { device.bind_image_memory(image, memory, 0)? };
        
        Ok(image)
    }
}
```

### 4. Performance Optimization Gaps

#### Problem Description
Current architecture lacks adaptive performance management and optimization strategies.

#### Missing Features
1. **Real-time Performance Monitoring**
2. **Adaptive Buffer Selection**
3. **Frame Rate Regulation**
4. **Memory Usage Optimization**

#### Solution: Adaptive Performance System
```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub frame_rate: f32,
    pub latency_ms: f32,
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub dropped_frames: u32,
}

pub struct AdaptivePerformanceManager {
    metrics: PerformanceMetrics,
    strategy: CaptureStrategy,
    performance_history: VecDeque<PerformanceMetrics>,
}

impl AdaptivePerformanceManager {
    pub fn update_strategy(&mut self, current_metrics: PerformanceMetrics) -> CaptureStrategy {
        self.performance_history.push_back(current_metrics.clone());
        self.metrics = current_metrics;
        
        // Adaptive strategy selection
        if self.metrics.frame_rate < 30.0 {
            // Performance critical - use SHM
            CaptureStrategy::PreferShm
        } else if self.metrics.frame_rate > 55.0 && self.metrics.latency_ms < 5.0 {
            // Performance good - use DMA
            CaptureStrategy::PreferDmabuf
        } else {
            // Maintain current strategy
            self.strategy.clone()
        }
    }
}
```

## 🔧 Implementation Roadmap

### Phase 1: Critical Issue Resolution (Week 1)
1. **Implement Software Rendering Fallback**
   - Add WGPU_BACKEND=gl support
   - Test with OpenGL backend
   - Verify basic functionality

2. **Fix Rust Environment**
   - Update SSH profile configuration
   - Verify consistent Rust version
   - Test remote builds

3. **Complete Surface Format Investigation**
   - Analyze Vulkan surface capabilities
   - Test format override approaches
   - Document compatible formats

### Phase 2: DMA Buffer Implementation (Week 2-3)
1. **GBM Integration**
   - Add GBM device management
   - Implement buffer allocation
   - Test DMA buffer creation

2. **Vulkan External Memory**
   - Add external memory extensions
   - Implement buffer import
   - Test GPU-to-GPU transfers

3. **Performance Optimization**
   - Add adaptive strategy selection
   - Implement performance monitoring
   - Test 60+ FPS achievement

### Phase 3: Production Readiness (Week 4)
1. **Error Handling**
   - Comprehensive error recovery
   - Graceful degradation
   - User-friendly error messages

2. **Configuration System**
   - Complete KDL integration
   - Runtime configuration updates
   - Performance tuning options

3. **Testing and Validation**
   - Multi-GPU testing
   - Stress testing
   - Performance benchmarking

## 🎯 Success Criteria

### Immediate Goals
- [ ] Application starts without crashing
- [ ] Basic rendering works (even with software fallback)
- [ ] Wayland screencopy connection established

### Short-term Goals
- [ ] 30+ FPS achieved with SHM buffers
- [ ] DMA buffer path functional
- [ ] Multi-screen display working

### Long-term Goals
- [ ] 60+ FPS sustained performance
- [ ] <2ms latency with DMA buffers
- [ ] Production-ready stability

---

**Link to Devin run**: https://app.devin.ai/sessions/d0f3ea092883490e904ec5a21c673b9c  
**Requested by**: @walue-dev  
**Last Updated**: July 30, 2025 14:46 UTC
