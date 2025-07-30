use bevy::prelude::*;
use bevy_niri_integration::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Niri Capture".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NiriCapturePlugin::default())
        .add_systems(Startup, setup_simple_scene)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(2.0)).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 1000.0,
            ..default()
        },
        Transform::from_translation(Vec3::new(2.0, 2.0, 2.0)),
    ));
}

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for mut transform in query.iter_mut() {
        let rotation = Quat::from_rotation_y(time.elapsed_secs() * 0.5);
        transform.translation = rotation * Vec3::new(0.0, 0.0, 3.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
