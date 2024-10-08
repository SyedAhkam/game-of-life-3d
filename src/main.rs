use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};
use bevy_flycam::prelude::*;
use itertools::iproduct;
use rand::seq::SliceRandom;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const WINDOW_RESOLUTION: (f32, f32) = (1920., 1080.);

const TIME_STEP: u64 = 500; // in millis

const PLANE_SIZE: i32 = 48;
const CANVAS_SIZE: i32 = 32;
const CELL_SIZE: i32 = 4;
const CELL_GAP: i32 = 1;
const _CELLS_ON_CANVAS: i32 = (CANVAS_SIZE / CELL_SIZE).pow(2);

const CELL_ALIVE_COLOR: Color = Color::srgb(0.9, 0., 0.);
const CELL_DEAD_COLOR: Color = Color::srgb(0.9, 0.9, 0.9); // or use Color::NONE to make dead cells disappear
const PLANE_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);

#[derive(Component, Debug, Clone, PartialEq)]
enum CellState {
    ALIVE,
    DEAD,
}

#[derive(Component, Debug, PartialEq)]
struct Position {
    x: i32,
    z: i32,
}

#[derive(Component, Debug)]
struct Neighbors(i64);

#[derive(Component, Debug)]
struct Cell;

#[derive(Bundle)]
struct CellBundle {
    pbr: PbrBundle,
    marker: Cell,
    state: CellState,
    position: Position,
    neighbors: Neighbors,
}

#[derive(Event, Default)]
struct RandomCellStates;

fn input_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_random_cell_state: EventWriter<RandomCellStates>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        info!("> regenerating cell states");

        ev_random_cell_state.send_default();
    }
}

fn cell_update(
    mut query: Query<(&Cell, &mut CellState, &Neighbors, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (_, mut state, neighbors, material_handle) in query.iter_mut() {
        let new_state = match *state {
            CellState::ALIVE => match neighbors.0 {
                0..=1 => CellState::DEAD,
                2..=3 => CellState::ALIVE,
                _ => CellState::DEAD,
            },
            CellState::DEAD => match neighbors.0 {
                3 => CellState::ALIVE,
                _ => CellState::DEAD,
            },
        };

        if new_state != *state {
            debug!("state changed: {:?} --> {:?}", *state, new_state);

            let material = materials.get_mut(material_handle).unwrap();

            material.base_color = match new_state {
                CellState::ALIVE => CELL_ALIVE_COLOR,
                CellState::DEAD => CELL_DEAD_COLOR,
            };
            *state = new_state;
        }
    }
}

fn count_neighbors(target_pos: &Position, query: &Query<(&Position, &CellState)>) -> Neighbors {
    let mut result = 0;

    let offset = CELL_SIZE + CELL_GAP;
    let possible_neighbor_positions = [
        // Up
        Position {
            x: target_pos.x + offset,
            z: target_pos.z,
        },
        // Up left
        Position {
            x: target_pos.x + offset,
            z: target_pos.z - offset,
        },
        // Up right
        Position {
            x: target_pos.x + offset,
            z: target_pos.z + offset,
        },
        // Bottom
        Position {
            x: target_pos.x - offset,
            z: target_pos.z,
        },
        // Bottom left
        Position {
            x: target_pos.x - offset,
            z: target_pos.z - offset,
        },
        // Bottom right
        Position {
            x: target_pos.x - offset,
            z: target_pos.z + offset,
        },
        // Left
        Position {
            x: target_pos.x,
            z: target_pos.z - offset,
        },
        // Right
        Position {
            x: target_pos.x,
            z: target_pos.z + offset,
        },
    ];

    // info!("{:?} -- {:?}", target_pos, possible_positions);

    for (pos, state) in query.iter() {
        if pos != target_pos
            && possible_neighbor_positions.contains(pos)
            && state == &CellState::ALIVE
        {
            result += 1;
        }
    }

    Neighbors(result)
}

fn neighbor_update(
    mut query: Query<(&Cell, &Position, &CellState, &mut Neighbors)>,
    readonly: Query<(&Position, &CellState)>,
) {
    for (_, pos, _, mut neighbors) in query.iter_mut() {
        let calculated = count_neighbors(pos, &readonly);

        *neighbors = calculated;
    }
}

fn random_cell_states(
    mut ev: EventReader<RandomCellStates>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Cell, &mut CellState, &Handle<StandardMaterial>)>,
) {
    for _ in ev.read() {
        let mut rng = rand::thread_rng();

        for (_, mut state, material_handle) in query.iter_mut() {
            let new_cell_state = [CellState::ALIVE, CellState::DEAD]
                .choose(&mut rng)
                .unwrap();

            let material = materials.get_mut(material_handle).unwrap();
            material.base_color = match new_cell_state {
                CellState::ALIVE => CELL_ALIVE_COLOR,
                CellState::DEAD => CELL_DEAD_COLOR,
            };

            *state = new_cell_state.to_owned();
        }
    }
}

fn setup_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_random_cell_state: EventWriter<RandomCellStates>,
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
            marker: Cell,
            state: CellState::DEAD,
            position: Position { x, z },
            neighbors: Neighbors(0),
        });
    }

    // After spawing all cells, issue event to randomly assign cell states
    ev_random_cell_state.send_default();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-(PLANE_SIZE * 2) as f32, 50.0, 0.)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::X),
            ..Default::default()
        },
        FlyCam,
    ));

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
    info!("Press R to regenerate cells");
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("{} v{}", APP_NAME, APP_VERSION),
                resolution: WINDOW_RESOLUTION.into(),
                present_mode: PresentMode::AutoVsync,
                mode: WindowMode::Fullscreen,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(Time::<Fixed>::from_duration(
            std::time::Duration::from_millis(TIME_STEP),
        ))
        .add_event::<RandomCellStates>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_cells)
        .add_systems(FixedUpdate, neighbor_update)
        .add_systems(FixedUpdate, cell_update.after(neighbor_update))
        .add_systems(Update, random_cell_states)
        .add_systems(Update, input_actions)
        .run();
}
