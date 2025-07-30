use std::collections::HashMap;
use crate::wayland_client::CaptureBuffer;

#[derive(Debug, Clone)]
pub struct AdaptiveCaptureSystem {
    performance_monitor: PerformanceMonitor,
    strategy: CaptureStrategy,
    output_stats: HashMap<String, OutputStats>,
}

#[derive(Debug, Clone)]
enum CaptureStrategy {
    PreferDmabuf,
    PreferShm,
    Adaptive { dmabuf_ratio: f32 },
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    frame_times: Vec<f32>,
    memory_usage: f32,
    gpu_utilization: f32,
}

#[derive(Debug, Clone)]
struct OutputStats {
    dmabuf_success_rate: f32,
    dmabuf_latency: f32,
    shm_latency: f32,
    memory_pressure: f32,
    recent_failures: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum CaptureMethod {
    Dmabuf,
    Shm,
}

impl AdaptiveCaptureSystem {
    pub fn new() -> Self {
        Self {
            performance_monitor: PerformanceMonitor::new(),
            strategy: CaptureStrategy::Adaptive { dmabuf_ratio: 0.8 },
            output_stats: HashMap::new(),
        }
    }
    
    pub fn select_optimal_method(&self, output: &str) -> CaptureMethod {
        let stats = self.output_stats.get(output)
            .cloned()
            .unwrap_or_else(|| OutputStats::default());
        
        match self.strategy {
            CaptureStrategy::PreferDmabuf => {
                if stats.dmabuf_success_rate > 0.7 {
                    CaptureMethod::Dmabuf
                } else {
                    CaptureMethod::Shm
                }
            }
            CaptureStrategy::PreferShm => CaptureMethod::Shm,
            CaptureStrategy::Adaptive { dmabuf_ratio } => {
                if stats.dmabuf_success_rate > 0.95 && stats.dmabuf_latency < 2.0 {
                    CaptureMethod::Dmabuf
                } else if stats.memory_pressure > 0.8 {
                    CaptureMethod::Dmabuf
                } else if stats.recent_failures > 3 {
                    CaptureMethod::Shm
                } else {
                    if rand::random::<f32>() < dmabuf_ratio {
                        CaptureMethod::Dmabuf
                    } else {
                        CaptureMethod::Shm
                    }
                }
            }
        }
    }
    
    pub fn record_capture_result(&mut self, output: &str, method: CaptureMethod, success: bool, latency: f32) {
        let stats = self.output_stats.entry(output.to_string())
            .or_insert_with(OutputStats::default);
        
        match method {
            CaptureMethod::Dmabuf => {
                if success {
                    stats.dmabuf_success_rate = stats.dmabuf_success_rate * 0.9 + 0.1;
                    stats.dmabuf_latency = stats.dmabuf_latency * 0.9 + latency * 0.1;
                    stats.recent_failures = 0;
                } else {
                    stats.dmabuf_success_rate = stats.dmabuf_success_rate * 0.9;
                    stats.recent_failures += 1;
                }
            }
            CaptureMethod::Shm => {
                if success {
                    stats.shm_latency = stats.shm_latency * 0.9 + latency * 0.1;
                } else {
                    stats.recent_failures += 1;
                }
            }
        }
        
        self.performance_monitor.record_frame_time(latency);
    }
    
    pub fn update_system_stats(&mut self, memory_usage: f32, gpu_utilization: f32) {
        self.performance_monitor.memory_usage = memory_usage;
        self.performance_monitor.gpu_utilization = gpu_utilization;
        
        for stats in self.output_stats.values_mut() {
            stats.memory_pressure = memory_usage;
        }
    }
    
    pub fn get_performance_summary(&self) -> String {
        format!(
            "Adaptive Capture Stats - Memory: {:.1}%, GPU: {:.1}%, Outputs: {}",
            self.performance_monitor.memory_usage * 100.0,
            self.performance_monitor.gpu_utilization * 100.0,
            self.output_stats.len()
        )
    }
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(60),
            memory_usage: 0.0,
            gpu_utilization: 0.0,
        }
    }
    
    fn record_frame_time(&mut self, frame_time: f32) {
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
    }
    
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        }
    }
}

impl Default for OutputStats {
    fn default() -> Self {
        Self {
            dmabuf_success_rate: 0.5,
            dmabuf_latency: 5.0,
            shm_latency: 10.0,
            memory_pressure: 0.0,
            recent_failures: 0,
        }
    }
}

impl Default for AdaptiveCaptureSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_system_creation() {
        let system = AdaptiveCaptureSystem::new();
        assert_eq!(system.output_stats.len(), 0);
    }
    
    #[test]
    fn test_method_selection() {
        let system = AdaptiveCaptureSystem::new();
        let method = system.select_optimal_method("test-output");
        
        match method {
            CaptureMethod::Dmabuf | CaptureMethod::Shm => {}
        }
    }
    
    #[test]
    fn test_performance_recording() {
        let mut system = AdaptiveCaptureSystem::new();
        system.record_capture_result("test", CaptureMethod::Dmabuf, true, 1.5);
        
        assert!(system.output_stats.contains_key("test"));
    }
}
