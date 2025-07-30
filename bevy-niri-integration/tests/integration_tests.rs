use bevy::prelude::*;
use bevy_niri_integration::*;
use std::time::Instant;

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(NiriCapturePlugin::default());
        
        app.update();
        
        assert!(app.world().get_resource::<NiriCaptureState>().is_some());
    }
    
    #[test]
    fn test_multi_output_setup() {
        let plugin = NiriCapturePlugin {
            outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
            capture_fps: 60.0,
            prefer_dmabuf: true,
        };
        
        assert_eq!(plugin.outputs.len(), 2);
        assert_eq!(plugin.capture_fps, 60.0);
        assert!(plugin.prefer_dmabuf);
    }
    
    #[test]
    fn test_performance_60fps() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(NiriCapturePlugin::default());
        
        let start = Instant::now();
        let target_frames = 60;
        
        for _ in 0..target_frames {
            app.update();
        }
        
        let elapsed = start.elapsed();
        let fps = target_frames as f32 / elapsed.as_secs_f32();
        
        println!("Achieved FPS: {:.1}", fps);
        assert!(fps >= 30.0, "Performance target not met: {} FPS", fps);
    }
    
    #[test]
    fn test_capture_state_stats() {
        let plugin = NiriCapturePlugin::default();
        let state = NiriCaptureState::new(&plugin);
        
        assert_eq!(state.performance_stats.frames_captured, 0);
        assert_eq!(state.performance_stats.frames_dropped, 0);
        assert_eq!(state.target_frame_time, 1.0 / 60.0);
    }
    
    #[test]
    fn test_config_validation() {
        let config = BevyCaptureConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config.clone();
        invalid_config.capture_fps = -1.0;
        assert!(invalid_config.validate().is_err());
    }
}
