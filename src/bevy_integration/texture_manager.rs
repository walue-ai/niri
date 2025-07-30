use std::collections::HashMap;
use std::time::Instant;

use bevy::prelude::*;
use bevy::asset::Handle;
use bevy::render::texture::Image;
use wgpu::TextureFormat;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use smithay::backend::renderer::{ExportMem, Renderer, Texture};
use smithay::backend::allocator::Fourcc;
use smithay::output::Output;
use smithay::utils::{Buffer, Size};

use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::renderer::{NiriRenderer, BevyCompatibleRenderer};
use super::renderer::BevyTexture;

#[derive(Debug)]
pub struct BevyTextureConverter {
    output_textures: HashMap<String, BevyTexture>,
    texture_cache: HashMap<String, Vec<u8>>,
}

impl BevyTextureConverter {
    pub fn new() -> Self {
        Self {
            output_textures: HashMap::new(),
            texture_cache: HashMap::new(),
        }
    }

    pub fn convert_texture<R>(
        &mut self,
        output: &Output,
        renderer: &mut R,
        texture_buffer: &TextureBuffer<R::NiriTextureId>,
    ) -> Result<BevyTexture, Box<dyn std::error::Error>> 
    where
        R: NiriRenderer + BevyCompatibleRenderer + ExportMem,
    {
        let texture = texture_buffer.texture();
        let size = texture.size();
        let output_name = output.name();
        
        let texture_data = self.export_texture_data(renderer, texture)?;
        
        let bevy_image = Image::new(
            Extent3d {
                width: size.w as u32,
                height: size.h as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data.clone(),
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        let handle = Handle::<Image>::default();
        
        let bevy_texture = BevyTexture {
            texture: handle,
            last_updated: Instant::now(),
            size: (size.w as u32, size.h as u32),
        };
        
        self.output_textures.insert(output_name.clone(), bevy_texture.clone());
        self.texture_cache.insert(output_name.clone(), texture_data);
        
        Ok(bevy_texture)
    }

    pub fn export_texture_data<R: NiriRenderer + ExportMem>(
        &self,
        renderer: &mut R,
        texture: &R::NiriTextureId,
    ) -> Result<Vec<u8>, R::Error> {
        let size = texture.size();
        let mut data = vec![0u8; (size.w * size.h * 4) as usize];
        
        data.fill(128); // Gray placeholder
        
        Ok(data)
    }

    pub fn add_output(&mut self, output: &Output) {
        debug!("BevyTextureConverter: Adding output {:?}", output.name());
    }

    pub fn remove_output(&mut self, output: &Output) {
        let output_name = output.name();
        self.output_textures.remove(&output_name);
        self.texture_cache.remove(&output_name);
        debug!("BevyTextureConverter: Removed output {:?}", output_name);
    }

    pub fn get_texture(&self, output: &Output) -> Option<&BevyTexture> {
        self.output_textures.get(&output.name())
    }

    pub fn update_texture_cache(&mut self, output: &Output, data: Vec<u8>) {
        self.texture_cache.insert(output.name(), data);
    }

    pub fn get_cached_texture_data(&self, output: &Output) -> Option<&Vec<u8>> {
        self.texture_cache.get(&output.name())
    }
}

impl Default for BevyTextureConverter {
    fn default() -> Self {
        Self::new()
    }
}
