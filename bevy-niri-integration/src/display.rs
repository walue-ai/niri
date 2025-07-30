use std::collections::HashMap;
use bevy::prelude::*;
use crate::plugin::{NiriCaptureState, NiriScreenDisplay};

pub fn setup_multi_screen_display(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    capture_state: Res<NiriCaptureState>,
) {
    let screen_count = capture_state.output_textures.len();
    let spacing = 2.5;
    
    info!("Setting up {} screen displays", screen_count);
    
    for (i, (output_name, texture_handle)) in capture_state.output_textures.iter().enumerate() {
        let x_offset = (i as f32 - (screen_count as f32 - 1.0) / 2.0) * spacing;
        
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(1.0)).mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture_handle.clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(x_offset, 0.0, 0.0),
            NiriScreenDisplay {
                output_name: output_name.clone(),
                auto_resize: true,
                damage_tracking: true,
            },
        ));
        
        commands.spawn((
            Text::new(format!("Niri Output: {}", output_name)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0 + i as f32 * 200.0),
                top: Val::Px(10.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        info!("Created display mesh for output: {} at x_offset: {}", output_name, x_offset);
    }
    
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 2.0, 5.0)),
    ));
}

pub fn update_screen_transforms(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<NiriScreenDisplay>>,
) {
    let rotation_speed = 0.2;
    let rotation = Quat::from_rotation_y(time.elapsed_secs() * rotation_speed);
    
    for mut transform in query.iter_mut() {
        transform.rotation = rotation;
    }
}

pub fn handle_screen_interaction(
    mut query: Query<(&mut Transform, &NiriScreenDisplay)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for (mut transform, display) in query.iter_mut() {
            transform.scale *= 1.1;
            info!("Scaled up display for output: {}", display.output_name);
        }
    }
    
    if input.just_pressed(KeyCode::KeyR) {
        for (mut transform, display) in query.iter_mut() {
            transform.scale = Vec3::ONE;
            transform.rotation = Quat::IDENTITY;
            info!("Reset transform for output: {}", display.output_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screen_display_component() {
        let display = NiriScreenDisplay {
            output_name: "test-output".to_string(),
            auto_resize: true,
            damage_tracking: true,
        };
        
        assert_eq!(display.output_name, "test-output");
        assert!(display.auto_resize);
        assert!(display.damage_tracking);
    }
}
