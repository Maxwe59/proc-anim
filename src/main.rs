use bevy::prelude::*;
mod mantis;
use mantis::create_mantis;
mod controls;
mod helper;
mod proc_anim;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use controls::controls_plugin;
use proc_anim::procedural_animation_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(controls_plugin)
        .add_plugins(procedural_animation_plugin)
        .insert_resource(WorldOptions {
            movement_mode: MovementMode::Legacy,
        })
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, update_fps_counter)
        .add_systems(Startup, setup)
        .add_systems(Startup, create_mantis)
        .add_systems(Startup, add_plane)
        //.add_plugins(EguiPlugin::default())
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}

#[derive(PartialEq, Debug)]
enum MovementMode {
    Mouse,
    Keyboard,
    Auto,
    Legacy,
}

#[derive(Resource)]
struct WorldOptions {
    movement_mode: MovementMode,
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn add_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

#[derive(Component)]
struct FpsText;

fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: "),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(1100.0),
            ..default()
        },
        FpsText,
    ));
}

fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("FPS: {:.0}", value);
            }
        }
    }
}
