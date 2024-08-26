use std::cell;

use bevy::{prelude::*, window::PresentMode};
use bevy_flycam::prelude::*;
use itertools::iproduct;
use rand::seq::SliceRandom;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const PLANE_SIZE: i32 = 48;
const CANVAS_SIZE: i32 = 32;
const CELL_SIZE: i32 = 4;
const CELL_GAP: i32 = 1;
const CELLS_ON_CANVAS: i32 = (CANVAS_SIZE / CELL_SIZE).pow(2);

const CELL_ALIVE_COLOR: Color = Color::srgb(0.8, 0.7, 0.6);
const CELL_DEAD_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PLANE_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);

#[derive(Component, Debug, Clone)]
enum CellState {
    ALIVE,
    DEAD,
}

#[derive(Component, Debug)]
struct Cell {
    x: i32,
    z: i32,
}

#[derive(Bundle)]
struct CellBundle {
    pbr: PbrBundle,
    marker: Cell,
    state: CellState,
}

fn state_update(cells_query: Query<(&Cell, &CellState), Changed<CellState>>) {
    // eprintln!("Cell state update");

    // let cells = cells_query.iter().collect::<Vec<&Cell>>();

    // for cell in cells {}
    // TODO
}

fn setup_states(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cells_query: Query<(&Cell, &mut CellState, &Handle<StandardMaterial>)>,
) {
    let mut rng = rand::thread_rng();

    for (cell, mut state, material_handle) in cells_query.iter() {
        let new_cell_state = [CellState::ALIVE, CellState::DEAD]
            .choose(&mut rng)
            .unwrap();

        let material = materials.get_mut(material_handle).unwrap();
        material.base_color = match new_cell_state {
            CellState::ALIVE => CELL_ALIVE_COLOR,
            CellState::DEAD => CELL_DEAD_COLOR,
        };

        state = new_cell_state;
    }
}

fn setup_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let neg_canvas = -CANVAS_SIZE;
    let cell_half_size = (CELL_SIZE / 2) as f32;

    for (x, z) in iproduct!(
        (neg_canvas..CANVAS_SIZE).step_by((CELL_SIZE + CELL_GAP) as usize),
        (neg_canvas..CANVAS_SIZE).step_by((CELL_SIZE + CELL_GAP) as usize)
    ) {
        commands.spawn(CellBundle {
            pbr: PbrBundle {
                mesh: meshes.add(Cuboid::from_length(CELL_SIZE as f32)),
                material: materials.add(CELL_DEAD_COLOR),
                transform: Transform::from_xyz(x as f32, cell_half_size, z as f32),
                ..Default::default()
            },
            marker: Cell { x, z },
            state: CellState::DEAD,
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
        mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(PLANE_SIZE as f32))),
        material: materials.add(PLANE_COLOR),
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
        .add_systems(Startup, setup_states.after(setup_cells))
        .add_systems(Update, state_update)
        .run();
}
