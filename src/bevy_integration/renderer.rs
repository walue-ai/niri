use std::time::Instant;

use bevy::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::render::texture::Image;
use wgpu::TextureFormat;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use smithay::output::Output;
use smithay::backend::renderer::{ExportMem, Texture};

use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::renderer::{NiriRenderer, BevyCompatibleRenderer};
use super::texture_manager::BevyTextureConverter;

pub struct BevyRenderer {
    app: App,
    texture_converter: BevyTextureConverter,
    initialized: bool,
    images: Assets<Image>,
}

#[derive(Debug, Clone)]
pub struct BevyTexture {
    pub texture: Handle<Image>,
    pub last_updated: Instant,
    pub size: (u32, u32),
}

impl BevyRenderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut app = App::new();
        
        app.add_plugins(MinimalPlugins);
        
        let images = Assets::<Image>::default();
        
        Ok(Self {
            app,
            texture_converter: BevyTextureConverter::new(),
            initialized: false,
            images,
        })
    }

    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            info!("Initializing Bevy renderer for niri integration");
            self.initialized = true;
        }
        Ok(())
    }

    pub fn update_from_niri_texture(
        &mut self,
        output: &Output,
        _renderer: &mut crate::backend::Backend,
        texture_data: &[u8],
    ) -> Result<BevyTexture, Box<dyn std::error::Error>> {
        let output_size = smithay::output::Output::current_mode(output)
            .map(|mode| (mode.size.w as u32, mode.size.h as u32))
            .unwrap_or((1920, 1080));
        
        let bevy_image = Image::new(
            Extent3d {
                width: output_size.0,
                height: output_size.1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            texture_data.to_vec(),
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );
        
        let handle = self.images.add(bevy_image);
        
        Ok(BevyTexture {
            texture: handle,
            last_updated: Instant::now(),
            size: output_size,
        })
    }

    pub fn handle_output_added(&mut self, output: &Output) {
        info!("Bevy renderer: Output added {:?}", output.name());
        self.texture_converter.add_output(output);
    }

    pub fn handle_output_removed(&mut self, output: &Output) {
        info!("Bevy renderer: Output removed {:?}", output.name());
        self.texture_converter.remove_output(output);
    }

    pub fn update(&mut self) {
        if self.initialized {
            self.app.update();
        }
    }

    pub fn get_output_texture(&self, output: &Output) -> Option<&BevyTexture> {
        self.texture_converter.get_texture(output)
    }
}

impl Default for BevyRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create BevyRenderer")
    }
}
