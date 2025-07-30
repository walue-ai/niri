use bevy::prelude::*;
use crate::wayland_client::CaptureBuffer;

pub fn format_buffer_info(buffer: &CaptureBuffer) -> String {
    match buffer {
        CaptureBuffer::Shm { width, height, stride, format, .. } => {
            format!("SHM Buffer: {}x{}, stride: {}, format: 0x{:x}", width, height, stride, format)
        }
        CaptureBuffer::Dmabuf { width, height, format, modifier, num_planes, .. } => {
            format!("DMA Buffer: {}x{}, format: 0x{:x}, modifier: 0x{:x}, planes: {}", 
                   width, height, format, modifier, num_planes)
        }
    }
}

pub fn calculate_texture_memory_usage(width: u32, height: u32, bytes_per_pixel: u32) -> u64 {
    (width as u64) * (height as u64) * (bytes_per_pixel as u64)
}

pub fn format_memory_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_calculation() {
        let usage = calculate_texture_memory_usage(1920, 1080, 4);
        assert_eq!(usage, 1920 * 1080 * 4);
    }
    
    #[test]
    fn test_memory_formatting() {
        assert_eq!(format_memory_size(1024), "1.00 KB");
        assert_eq!(format_memory_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_memory_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
