use bevy::prelude::*;
use bevy_niri_integration::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Niri Multi-Screen Display".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NiriCapturePlugin {
            outputs: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
            capture_fps: 60.0,
            prefer_dmabuf: true,
        })
        .add_systems(Startup, (
            setup_camera_system,
            setup_ui_system,
        ))
        .add_systems(Update, (
            handle_input_system,
            update_performance_display,
            bevy_niri_integration::display::handle_screen_interaction,
            bevy_niri_integration::display::update_screen_transforms,
        ))
        .add_systems(PostStartup, bevy_niri_integration::display::setup_multi_screen_display)
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    info!("Camera system initialized");
}

fn setup_ui_system(mut commands: Commands) {
    commands.spawn((
        Text::from_section(
            "Niri Multi-Screen Display\nPress SPACE to scale, R to reset, ESC to exit",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            }
        ),
        Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
    
    commands.spawn((
        Text::from_section(
            "Performance: Initializing...",
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(1.0, 1.0, 0.0),
                ..default()
            }
        ),
        Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        PerformanceDisplay,
    ));
}

#[derive(Component)]
struct PerformanceDisplay;

fn handle_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
        info!("Exit requested by user");
    }
}

fn update_performance_display(
    mut query: Query<&mut Text, With<PerformanceDisplay>>,
    capture_state: Res<NiriCaptureState>,
    time: Res<Time>,
) {
    for mut text in query.iter_mut() {
        let fps = 1.0 / time.delta_seconds();
        let stats = &capture_state.performance_stats;
        
        text.sections[0].value = format!(
            "Performance: {:.1} FPS | Captured: {} | Dropped: {} | Latency: {:.1}ms",
            fps,
            stats.frames_captured,
            stats.frames_dropped,
            stats.average_latency * 1000.0
        );
    }
}
