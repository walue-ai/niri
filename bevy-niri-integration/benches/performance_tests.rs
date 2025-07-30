use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy::prelude::*;
use bevy_niri_integration::*;

fn benchmark_capture_system(c: &mut Criterion) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(NiriCapturePlugin::default());
    
    c.bench_function("capture_system_update", |b| {
        b.iter(|| {
            app.update();
            black_box(&app);
        })
    });
}

fn benchmark_texture_conversion(c: &mut Criterion) {
    use bevy_niri_integration::dma_integration::DmaBufferConverter;
    use bevy_niri_integration::wayland_client::CaptureBuffer;
    
    let converter = DmaBufferConverter::new();
    let mut images = Assets::<Image>::default();
    
    let shm_buffer = CaptureBuffer::Shm {
        data: vec![255, 0, 0, 255; 1920 * 1080],
        stride: 1920 * 4,
        width: 1920,
        height: 1080,
        format: 0x34325258,
    };
    
    c.bench_function("shm_to_texture_conversion", |b| {
        b.iter(|| {
            let result = converter.convert_shm_to_bevy_texture(black_box(&shm_buffer), &mut images);
            black_box(result);
        })
    });
}

fn benchmark_adaptive_selection(c: &mut Criterion) {
    use bevy_niri_integration::adaptive::{AdaptiveCaptureSystem, CaptureMethod};
    
    let mut system = AdaptiveCaptureSystem::new();
    
    system.record_capture_result("test-output", CaptureMethod::Dmabuf, true, 1.5);
    system.record_capture_result("test-output", CaptureMethod::Shm, true, 8.0);
    
    c.bench_function("adaptive_method_selection", |b| {
        b.iter(|| {
            let method = system.select_optimal_method(black_box("test-output"));
            black_box(method);
        })
    });
}

criterion_group!(
    benches,
    benchmark_capture_system,
    benchmark_texture_conversion,
    benchmark_adaptive_selection
);
criterion_main!(benches);
