use bevy::{prelude::*, utils::HashMap, window::PresentMode};
use bevy_flycam::prelude::*;
use itertools::iproduct;
use rand::seq::SliceRandom;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const WINDOW_RESOLUTION: (f32, f32) = (1920., 1080.);

const TIME_STEP: u64 = 200; // in millis

const PLANE_SIZE: i32 = 0;
const CANVAS_SIZE: i32 = 2i32.pow(7); // Reduced from 2^8 to 2^7 for better performance
const CELL_SIZE: i32 = 4;
const CELL_GAP: i32 = 1;
const _CELLS_ON_CANVAS: i32 = (CANVAS_SIZE / CELL_SIZE).pow(2);

const CELL_ALIVE_COLOR: Color = Color::srgb(0.9, 0., 0.);
const CELL_DEAD_COLOR: Color = Color::srgb(0.9, 0.9, 0.9); // or use Color::NONE to make dead cells disappear
const PLANE_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum CellState {
    ALIVE,
    DEAD,
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct GridPos {
    x: i32,
    z: i32,
}

#[derive(Component, Debug)]
struct Cell;

#[derive(Bundle)]
struct CellBundle {
    pbr: PbrBundle,
    marker: Cell,
    state: CellState,
    grid_pos: GridPos,
}

#[derive(Event, Default)]
struct RandomCellStates;

#[derive(Resource)]
struct CellGrid {
    // HashMap for O(1) lookups - maps grid position to (entity, current_state, next_state)
    cells: HashMap<GridPos, (Entity, CellState, CellState)>,
    // Pre-calculated neighbor offsets
    neighbor_offsets: Vec<GridPos>,
}

impl Default for CellGrid {
    fn default() -> Self {
        let _offset = (CELL_SIZE + CELL_GAP) / (CELL_SIZE + CELL_GAP); // This should be 1 in grid coordinates
        let neighbor_offsets = vec![
            GridPos { x: -1, z: -1 }, // Bottom left
            GridPos { x: -1, z: 0 },  // Bottom
            GridPos { x: -1, z: 1 },  // Bottom right
            GridPos { x: 0, z: -1 },  // Left
            GridPos { x: 0, z: 1 },   // Right
            GridPos { x: 1, z: -1 },  // Top left
            GridPos { x: 1, z: 0 },   // Top
            GridPos { x: 1, z: 1 },   // Top right
        ];

        Self {
            cells: HashMap::new(),
            neighbor_offsets,
        }
    }
}

fn input_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_random_cell_state: EventWriter<RandomCellStates>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        info!("> regenerating cell states");
        ev_random_cell_state.send_default();
    }
}

fn world_pos_to_grid_pos(world_x: i32, world_z: i32) -> GridPos {
    let step = CELL_SIZE + CELL_GAP;
    GridPos {
        x: world_x / step,
        z: world_z / step,
    }
}

fn count_alive_neighbors(pos: GridPos, grid: &CellGrid) -> u8 {
    let mut count = 0;

    for offset in &grid.neighbor_offsets {
        let neighbor_pos = GridPos {
            x: pos.x + offset.x,
            z: pos.z + offset.z,
        };

        if let Some((_, state, _)) = grid.cells.get(&neighbor_pos) {
            if *state == CellState::ALIVE {
                count += 1;
            }
        }
    }

    count
}

// First pass: calculate next states for all cells
fn calculate_next_states(mut grid: ResMut<CellGrid>) {
    // Create a temporary map to store the next states
    let mut next_states: HashMap<GridPos, CellState> = HashMap::new();

    // First, calculate all next states
    for (&pos, (_, current_state, _)) in &grid.cells {
        let alive_neighbors = count_alive_neighbors(pos, &grid);

        let new_state = match *current_state {
            CellState::ALIVE => match alive_neighbors {
                0..=1 => CellState::DEAD,
                2..=3 => CellState::ALIVE,
                _ => CellState::DEAD,
            },
            CellState::DEAD => match alive_neighbors {
                3 => CellState::ALIVE,
                _ => CellState::DEAD,
            },
        };

        next_states.insert(pos, new_state);
    }

    // Then, update the next_state in the grid
    for (pos, new_state) in next_states {
        if let Some((_, _, next_state)) = grid.cells.get_mut(&pos) {
            *next_state = new_state;
        }
    }
}

// Second pass: apply state changes and update materials
fn apply_state_changes(
    mut grid: ResMut<CellGrid>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, With<Cell>>,
) {
    let mut changes = 0;

    for (entity, current_state, next_state) in grid.cells.values_mut() {
        if *current_state != *next_state {
            if let Ok(material_handle) = query.get(*entity) {
                if let Some(material) = materials.get_mut(material_handle) {
                    material.base_color = match *next_state {
                        CellState::ALIVE => CELL_ALIVE_COLOR,
                        CellState::DEAD => CELL_DEAD_COLOR,
                    };
                    changes += 1;
                }
            }
            *current_state = *next_state;
        }
    }

    if changes > 0 {
        debug!("Updated {} cell materials", changes);
    }
}

fn random_cell_states(
    mut ev: EventReader<RandomCellStates>,
    mut grid: ResMut<CellGrid>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Handle<StandardMaterial>, With<Cell>>,
) {
    for _ in ev.read() {
        let mut rng = rand::thread_rng();
        let mut updated_count = 0;

        for (entity, current_state, next_state) in grid.cells.values_mut() {
            let new_cell_state = [CellState::ALIVE, CellState::DEAD]
                .choose(&mut rng)
                .unwrap();

            if let Ok(material_handle) = query.get(*entity) {
                if let Some(material) = materials.get_mut(material_handle) {
                    material.base_color = match new_cell_state {
                        CellState::ALIVE => CELL_ALIVE_COLOR,
                        CellState::DEAD => CELL_DEAD_COLOR,
                    };
                    updated_count += 1;
                }
            }

            *current_state = new_cell_state.clone();
            *next_state = new_cell_state.clone();
        }

        info!("Randomized {} cells", updated_count);
    }
}

fn setup_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<CellGrid>,
    mut ev_random_cell_state: EventWriter<RandomCellStates>,
) {
    let neg_canvas = -CANVAS_SIZE;
    let cell_half_size = (CELL_SIZE / 2) as f32;
    let step = CELL_SIZE + CELL_GAP;

    // Pre-allocate HashMap capacity for better performance
    let expected_cells = ((CANVAS_SIZE * 2) / step).pow(2) as usize;
    grid.cells.reserve(expected_cells);

    for (world_x, world_z) in iproduct!(
        (neg_canvas..CANVAS_SIZE).step_by(step as usize),
        (neg_canvas..CANVAS_SIZE).step_by(step as usize)
    ) {
        let grid_pos = world_pos_to_grid_pos(world_x, world_z);

        let entity = commands
            .spawn(CellBundle {
                pbr: PbrBundle {
                    mesh: meshes.add(Cuboid::from_length(CELL_SIZE as f32)),
                    material: materials.add(CELL_DEAD_COLOR),
                    transform: Transform::from_xyz(world_x as f32, cell_half_size, world_z as f32),
                    ..Default::default()
                },
                marker: Cell,
                state: CellState::DEAD,
                grid_pos,
            })
            .id();

        grid.cells
            .insert(grid_pos, (entity, CellState::DEAD, CellState::DEAD));
    }

    info!("Created {} cells", grid.cells.len());

    // After spawning all cells, issue event to randomly assign cell states
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
            transform: Transform::from_xyz(-(PLANE_SIZE * 2) as f32, 400.0, 0.)
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
                // mode: WindowMode::Fullscreen,
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
        .init_resource::<CellGrid>()
        .add_event::<RandomCellStates>()
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_cells)
        .add_systems(
            FixedUpdate,
            (
                calculate_next_states,
                apply_state_changes.after(calculate_next_states),
            ),
        )
        .add_systems(Update, random_cell_states)
        .add_systems(Update, input_actions)
        .run();
}
