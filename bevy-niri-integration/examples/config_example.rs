use bevy::prelude::*;
use bevy_niri_integration::*;

fn main() {
    let config = BevyCaptureConfig {
        capture_fps: 120.0,
        prefer_dmabuf: true,
        outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
        damage_tracking: true,
        adaptive_performance: true,
        max_memory_usage: 0.7,
    };
    
    println!("Configuration validation: {:?}", config.validate());
    println!("KDL representation:\n{}", config.to_kdl_string());
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NiriCapturePlugin {
            outputs: config.outputs.clone(),
            capture_fps: config.capture_fps,
            prefer_dmabuf: config.prefer_dmabuf,
        })
        .add_systems(Startup, setup_demo_scene)
        .run();
}

fn setup_demo_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.0),
    ));
    
    info!("Demo scene with custom configuration initialized");
}
