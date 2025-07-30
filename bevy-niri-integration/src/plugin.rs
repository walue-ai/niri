use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureUsages};
use crate::wayland_client::NiriScreencopyClient;

#[derive(Debug, Clone)]
pub struct NiriCapturePlugin {
    pub outputs: Vec<String>,
    pub capture_fps: f32,
    pub prefer_dmabuf: bool,
}

impl Default for NiriCapturePlugin {
    fn default() -> Self {
        Self {
            outputs: vec![],
            capture_fps: 60.0,
            prefer_dmabuf: true,
        }
    }
}

#[derive(Resource)]
pub struct NiriCaptureState {
    pub client: Option<NiriScreencopyClient>,
    pub output_textures: HashMap<String, Handle<Image>>,
    pub performance_stats: CaptureStats,
    pub last_capture_time: f64,
    pub target_frame_time: f64,
}

#[derive(Component)]
pub struct NiriScreenDisplay {
    pub output_name: String,
    pub auto_resize: bool,
    pub damage_tracking: bool,
}

#[derive(Debug, Default)]
pub struct CaptureStats {
    pub frames_captured: u64,
    pub frames_dropped: u64,
    pub average_latency: f32,
    pub dmabuf_success_rate: f32,
    pub shm_fallback_count: u64,
}

impl NiriCaptureState {
    pub fn new(plugin_config: &NiriCapturePlugin) -> Self {
        let client = NiriScreencopyClient::new().ok();
        
        Self {
            client,
            output_textures: HashMap::new(),
            performance_stats: CaptureStats::default(),
            last_capture_time: 0.0,
            target_frame_time: 1.0 / plugin_config.capture_fps as f64,
        }
    }
}

impl Plugin for NiriCapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NiriCaptureState::new(self))
            .add_systems(Startup, setup_niri_capture_system)
            .add_systems(Update, (
                capture_screens_system,
                update_textures_system,
                performance_monitoring_system,
            ));
    }
}

fn setup_niri_capture_system(
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut capture_state: ResMut<NiriCaptureState>,
) {
    if let Some(ref client) = capture_state.client {
        let outputs = client.get_outputs();
        
        for output in outputs {
            let image = images.add({
                let size = Extent3d {
                    width: output.width,
                    height: output.height,
                    depth_or_array_layers: 1,
                };
                let mut image = Image {
                    data: vec![0; (size.width * size.height * 4) as usize],
                    ..default()
                };
                image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
                image.texture_descriptor.size = size;
                image
            });
            
            capture_state.output_textures.insert(output.name.clone(), image.clone());
            
            info!("Created texture for output: {} ({}x{})", output.name, output.width, output.height);
        }
    } else {
        warn!("Failed to initialize Niri screencopy client");
    }
}

fn capture_screens_system(
    time: Res<Time>,
    mut capture_state: ResMut<NiriCaptureState>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    if current_time - capture_state.last_capture_time < capture_state.target_frame_time {
        return;
    }
    
    let outputs: Vec<String> = capture_state.output_textures.keys().cloned().collect();
    let mut frames_captured = 0;
    let mut frames_dropped = 0;
    
    if let Some(ref mut client) = capture_state.client {
        for output_name in outputs {
            match client.capture_output(&output_name) {
                Ok(_buffer) => {
                    frames_captured += 1;
                    trace!("Captured frame for output: {}", output_name);
                }
                Err(e) => {
                    frames_dropped += 1;
                    warn!("Failed to capture output {}: {:?}", output_name, e);
                }
            }
        }
        
        if let Err(e) = client.poll_events() {
            warn!("Failed to poll Wayland events: {:?}", e);
        }
    }
    
    capture_state.performance_stats.frames_captured += frames_captured;
    capture_state.performance_stats.frames_dropped += frames_dropped;
    capture_state.last_capture_time = current_time;
}

fn update_textures_system(
    mut images: ResMut<Assets<Image>>,
    capture_state: Res<NiriCaptureState>,
) {
    for (_output_name, texture_handle) in &capture_state.output_textures {
        if let Some(image) = images.get_mut(texture_handle) {
            let data = &mut image.data;
                let pattern = ((capture_state.performance_stats.frames_captured % 255) as u8, 
                              ((capture_state.performance_stats.frames_captured / 2) % 255) as u8,
                              ((capture_state.performance_stats.frames_captured / 4) % 255) as u8);
                
            for chunk in data.chunks_mut(4) {
                if chunk.len() >= 4 {
                    chunk[0] = pattern.0;
                    chunk[1] = pattern.1;
                    chunk[2] = pattern.2;
                    chunk[3] = 255;
                }
            }
        }
    }
}

fn performance_monitoring_system(
    time: Res<Time>,
    mut capture_state: ResMut<NiriCaptureState>,
) {
    let fps = 1.0 / time.delta_seconds();
    
    if fps < 60.0 {
        capture_state.performance_stats.frames_dropped += 1;
        debug!("Performance warning: FPS dropped to {:.1}", fps);
    }
    
    capture_state.performance_stats.average_latency = 
        (capture_state.performance_stats.average_latency * 0.9) + (time.delta_seconds() * 0.1);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_default() {
        let plugin = NiriCapturePlugin::default();
        assert_eq!(plugin.capture_fps, 60.0);
        assert!(plugin.prefer_dmabuf);
        assert!(plugin.outputs.is_empty());
    }
    
    #[test]
    fn test_capture_state_creation() {
        let plugin = NiriCapturePlugin::default();
        let state = NiriCaptureState::new(&plugin);
        
        assert_eq!(state.target_frame_time, 1.0 / 60.0);
        assert_eq!(state.performance_stats.frames_captured, 0);
    }
}
