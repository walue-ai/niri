use std::time::Instant;

use bevy::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::render::texture::Image;
use wgpu::TextureFormat;
use bevy::render::render_resource::{Extent3d, TextureDimension};
use bevy::pbr::{StandardMaterial, PbrBundle, PointLight, PointLightBundle};
use bevy::ui::{Style, PositionType, Val};
use bevy::prelude::{TextBundle, TextStyle, Text};
use smithay::output::Output;
use smithay::backend::renderer::{ExportMem, Texture};

use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::renderer::{NiriRenderer, BevyCompatibleRenderer};
use super::texture_manager::BevyTextureConverter;

#[derive(Component)]
struct RotatingCube;

#[derive(Resource)]
struct BevyExampleTimer(Timer);

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
        
        app.add_plugins((
            bevy::core::TaskPoolPlugin::default(),
            bevy::core::TypeRegistrationPlugin,
            bevy::core::FrameCountPlugin,
            bevy::time::TimePlugin,
            bevy::transform::TransformPlugin,
            bevy::hierarchy::HierarchyPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::render::RenderPlugin::default(),
            bevy::render::texture::ImagePlugin::default(),
            bevy::pbr::PbrPlugin::default(),
            bevy::ui::UiPlugin,
            bevy::text::TextPlugin,
        ));
        
        app.add_systems(Startup, setup_demo_scene);
        app.add_systems(Update, (rotate_cube, update_demo_text));
        app.insert_resource(BevyExampleTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        
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

fn setup_demo_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        RotatingCube,
    ));

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb_u8(63, 124, 9)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(
        TextBundle::from_section(
            "Niri-Bevy Integration Demo\nWayland Client Testing\nBevy renderer active!",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn rotate_cube(mut query: Query<&mut Transform, With<RotatingCube>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * 0.5);
        transform.rotate_x(time.delta_seconds() * 0.3);
    }
}

fn update_demo_text(
    mut timer: ResMut<BevyExampleTimer>,
    time: Res<Time>,
    mut query: Query<&mut Text>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut text in &mut query {
            if let Some(section) = text.sections.first_mut() {
                let elapsed = time.elapsed_seconds() as u32;
                section.value = format!(
                    "Niri-Bevy Integration Demo\nWayland Client Testing\nBevy renderer active!\nRuntime: {}s",
                    elapsed
                );
            }
        }
    }
}

impl Default for BevyRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create BevyRenderer")
    }
}
