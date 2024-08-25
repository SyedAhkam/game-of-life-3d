use std::cell;

use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::prelude::*;
use itertools::iproduct;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const CANVAS_SIZE: i32 = 32;
const CELL_SIZE: i32 = 4;
const CELLS_ON_CANVAS: i32 = (CANVAS_SIZE / CELL_SIZE).pow(2);

const CELL_COLOR: Color = Color::srgb(0.8, 0.7, 0.6);
const CANVAS_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);

fn setup_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let neg_canvas = -CANVAS_SIZE;
    let cell_half_size = (CELL_SIZE / 2) as f32;
    for (x, z) in iproduct!(
        (neg_canvas..CANVAS_SIZE).step_by(CELL_SIZE as usize),
        (neg_canvas..CANVAS_SIZE).step_by(CELL_SIZE as usize)
    ) {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::from_length(CELL_SIZE as f32)),
            material: materials.add(CELL_COLOR),
            transform: Transform::from_xyz(x as f32, cell_half_size, z as f32),
            ..Default::default()
        });
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(CANVAS_SIZE as f32))),
        material: materials.add(CANVAS_COLOR),
        ..Default::default()
    },));

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
        .add_systems(Startup, setup_cells)
        .run();
}
