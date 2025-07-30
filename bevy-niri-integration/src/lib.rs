pub mod wayland_client;
pub mod plugin;
pub mod display;
pub mod dma_integration;
pub mod adaptive;
pub mod config;
pub mod error;
pub mod utils;

pub use plugin::{NiriCapturePlugin, NiriCaptureState, NiriScreenDisplay};
pub use wayland_client::{NiriScreencopyClient, OutputInfo, CaptureBuffer};
pub use config::BevyCaptureConfig;
pub use error::{CaptureError, ConversionError};
pub use utils::*;

use bevy::prelude::*;

pub struct NiriIntegrationPlugin;

impl Plugin for NiriIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NiriCapturePlugin::default());
    }
}
