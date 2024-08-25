use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::prelude::*;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const PLANE_SIZE: f32 = 64.;
const CUBE_SIZE: f32 = 4.;

const CUBE_COLOR: Color = Color::srgb(0.8, 0.7, 0.6);
const PLANE_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: APP_NAME.into(),
                resolution: (700., 500.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                // fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                // prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(PLANE_SIZE))),
        material: materials.add(PLANE_COLOR),
        ..Default::default()
    },));

    // Cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_length(CUBE_SIZE)),
        material: materials.add(CUBE_COLOR),
        transform: Transform::from_xyz(0., CUBE_SIZE / 2., 0.),
        ..Default::default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 127_000_000.,
            range: 100.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 20., 0.),
        ..Default::default()
    });

    info!("Move camera around by using WASD for lateral movement");
    info!("Use Left Shift and Spacebar for vertical movement");
    info!("Use the mouse to look around");
    info!("Press Esc to hide or show the mouse cursor");
}
