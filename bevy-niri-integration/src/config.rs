use serde::{Deserialize, Serialize};
use crate::plugin::NiriCapturePlugin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BevyCaptureConfig {
    pub capture_fps: f32,
    pub prefer_dmabuf: bool,
    pub outputs: Vec<String>,
    pub damage_tracking: bool,
    pub adaptive_performance: bool,
    pub max_memory_usage: f32,
}

impl Default for BevyCaptureConfig {
    fn default() -> Self {
        Self {
            capture_fps: 60.0,
            prefer_dmabuf: true,
            outputs: vec![],
            damage_tracking: true,
            adaptive_performance: true,
            max_memory_usage: 0.8,
        }
    }
}

impl BevyCaptureConfig {
    pub fn from_kdl_string(kdl_content: &str) -> Result<Self, String> {
        let mut config = Self::default();
        
        for line in kdl_content.lines() {
            let line = line.trim();
            if line.starts_with("bevy-capture") {
                if line.contains("capture-fps") {
                    if let Some(fps_str) = line.split('=').nth(1) {
                        if let Ok(fps) = fps_str.trim().parse::<f32>() {
                            config.capture_fps = fps;
                        }
                    }
                }
                if line.contains("prefer-dmabuf") {
                    if let Some(dmabuf_str) = line.split('=').nth(1) {
                        config.prefer_dmabuf = dmabuf_str.trim() == "true";
                    }
                }
                if line.contains("damage-tracking") {
                    if let Some(damage_str) = line.split('=').nth(1) {
                        config.damage_tracking = damage_str.trim() == "true";
                    }
                }
            }
        }
        
        Ok(config)
    }
    
    pub fn merge_with_api(&mut self, api_config: &NiriCapturePlugin) {
        if api_config.capture_fps > 0.0 {
            self.capture_fps = api_config.capture_fps;
        }
        
        self.prefer_dmabuf = api_config.prefer_dmabuf;
        
        if !api_config.outputs.is_empty() {
            self.outputs = api_config.outputs.clone();
        }
    }
    
    pub fn to_kdl_string(&self) -> String {
        format!(
            r#"bevy-capture {{
    capture-fps = {}
    prefer-dmabuf = {}
    damage-tracking = {}
    adaptive-performance = {}
    max-memory-usage = {}
}}"#,
            self.capture_fps,
            self.prefer_dmabuf,
            self.damage_tracking,
            self.adaptive_performance,
            self.max_memory_usage
        )
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.capture_fps <= 0.0 || self.capture_fps > 240.0 {
            return Err("capture_fps must be between 0 and 240".to_string());
        }
        
        if self.max_memory_usage <= 0.0 || self.max_memory_usage > 1.0 {
            return Err("max_memory_usage must be between 0 and 1".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = BevyCaptureConfig::default();
        assert_eq!(config.capture_fps, 60.0);
        assert!(config.prefer_dmabuf);
        assert!(config.damage_tracking);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = BevyCaptureConfig::default();
        assert!(config.validate().is_ok());
        
        config.capture_fps = -1.0;
        assert!(config.validate().is_err());
        
        config.capture_fps = 60.0;
        config.max_memory_usage = 1.5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_kdl_parsing() {
        let kdl_content = r#"
            bevy-capture capture-fps = 120
            bevy-capture prefer-dmabuf = false
        "#;
        
        let config = BevyCaptureConfig::from_kdl_string(kdl_content).unwrap();
        assert_eq!(config.capture_fps, 120.0);
        assert!(!config.prefer_dmabuf);
    }
    
    #[test]
    fn test_api_merge() {
        let mut config = BevyCaptureConfig::default();
        let api_config = NiriCapturePlugin {
            outputs: vec!["DP-1".to_string()],
            capture_fps: 120.0,
            prefer_dmabuf: false,
        };
        
        config.merge_with_api(&api_config);
        assert_eq!(config.capture_fps, 120.0);
        assert!(!config.prefer_dmabuf);
        assert_eq!(config.outputs, vec!["DP-1"]);
    }
}
