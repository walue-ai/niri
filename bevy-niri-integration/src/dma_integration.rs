use std::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureUsages, TextureFormat, TextureDimension};
use crate::wayland_client::CaptureBuffer;
use crate::error::ConversionError;

pub struct DmaBufferConverter {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl DmaBufferConverter {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
        }
    }
    
    pub fn initialize(&mut self, device: wgpu::Device, queue: wgpu::Queue) {
        self.device = Some(device);
        self.queue = Some(queue);
    }
    
    pub fn convert_dmabuf_to_bevy_texture(
        &self,
        dmabuf_buffer: &CaptureBuffer,
        images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, ConversionError> {
        match dmabuf_buffer {
            CaptureBuffer::Dmabuf { width, height, format, .. } => {
                let image = Image::new(
                    Extent3d {
                        width: *width,
                        height: *height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    vec![0; (*width * *height * 4) as usize],
                    TextureFormat::Rgba8UnormSrgb,
                    bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
                );
                
                Ok(images.add(image))
            }
            _ => Err(ConversionError::UnsupportedFormat("Expected DMA buffer".to_string())),
        }
    }
    
    pub fn convert_shm_to_bevy_texture(
        &self,
        shm_buffer: &CaptureBuffer,
        images: &mut Assets<Image>,
    ) -> Result<Handle<Image>, ConversionError> {
        match shm_buffer {
            CaptureBuffer::Shm { data, width, height, stride, format } => {
                let mut rgba_data = Vec::with_capacity((*width * *height * 4) as usize);
                
                for y in 0..*height {
                    for x in 0..*width {
                        let src_offset = (y * stride + x * 4) as usize;
                        if src_offset + 3 < data.len() {
                            rgba_data.push(data[src_offset + 2]); // R
                            rgba_data.push(data[src_offset + 1]); // G
                            rgba_data.push(data[src_offset + 0]); // B
                            rgba_data.push(255);                  // A
                        } else {
                            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
                        }
                    }
                }
                
                let image = Image::new(
                    Extent3d {
                        width: *width,
                        height: *height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    rgba_data,
                    TextureFormat::Rgba8UnormSrgb,
                    bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
                );
                
                Ok(images.add(image))
            }
            _ => Err(ConversionError::UnsupportedFormat("Expected SHM buffer".to_string())),
        }
    }
    
    pub fn supports_dmabuf(&self) -> bool {
        self.device.is_some()
    }
}

impl Default for DmaBufferConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_converter_creation() {
        let converter = DmaBufferConverter::new();
        assert!(!converter.supports_dmabuf());
    }
    
    #[test]
    fn test_shm_conversion() {
        let converter = DmaBufferConverter::new();
        let mut images = Assets::<Image>::default();
        
        let shm_buffer = CaptureBuffer::Shm {
            data: vec![255, 0, 0, 255; 1920 * 1080],
            stride: 1920 * 4,
            width: 1920,
            height: 1080,
            format: 0x34325258,
        };
        
        let result = converter.convert_shm_to_bevy_texture(&shm_buffer, &mut images);
        assert!(result.is_ok());
    }
}
