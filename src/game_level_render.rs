use crate::game_level::{GameLevel, HALF_TILE_SIZE, TILE_SIZE};
use crate::grid::GridPosition;
use crate::prelude::*;
use crate::tasks::{Minable, Standable};
use crate::{GameState, MyAssets};
use oxidized_navigation::NavMeshAffector;
use std::collections::HashMap;

pub struct GameLevelRenderPlugin;

impl Plugin for GameLevelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            spawn_map_content
                .run_if(resource_exists::<MyAssets>())
                .run_if(resource_exists::<GameLevel>())
                .run_if(in_state(GameState::Playing)),
        )
        .insert_resource(WorldTileTracker::default());
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct WorldTilePosition {
    x: i32,
    z: i32,
}

#[derive(Resource, Default)]
struct WorldTileTracker {
    tiles: HashMap<WorldTilePosition, WorldTile>,
}

struct WorldTile {
    wall_entity: Option<Entity>,
    floor_entity: Entity,
}

fn spawn_map_content(
    level: Res<GameLevel>,
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    mesh_assets: Res<Assets<Mesh>>,
    mut world_tile_tracker: ResMut<WorldTileTracker>,
) {
    let mut updated_positions = Vec::new();

    for x in -1..=level.width() + 1 {
        for z in -1..=level.height() + 1 {
            let world_tile_position = WorldTilePosition { x, z };
            let mut world_tile_entry = world_tile_tracker.tiles.get_mut(&world_tile_position);

            if let Some(ref mut existing) = world_tile_entry {
                if let Some(wall_entity) = existing.wall_entity {
                    if level.is_open(x, z) {
                        if let Some(wall_entity) = commands.get_entity(wall_entity) {
                            info!("Removing wall entity");
                            wall_entity.despawn_recursive();
                        }
                        existing.wall_entity = None;
                        updated_positions.push(world_tile_position);
                    }
                }
            } else {
                let grid_position = GridPosition::new(x, z);
                let pos = level.get_position_at(grid_position);
                let mut wall_entity = None;
                if let Some((wall_mesh, rotation)) =
                    get_wall_mesh(&*level, &grid_position, &*my_assets)
                {
                    let collider = if let Some(mesh) = mesh_assets.get(&wall_mesh) {
                        Collider::from_bevy_mesh(&mesh, &default())
                            .expect("Failed to create collider from mesh")
                    } else {
                        error!("Failed to get mesh {:?}", wall_mesh);
                        Collider::cuboid(5., 5., 5.)
                    };

                    let id = commands
                        .spawn((
                            PbrBundle {
                                transform: Transform::from_xyz(pos.x, HALF_TILE_SIZE, pos.z)
                                    .with_rotation(rotation),
                                material: my_assets.wall_material.clone(),
                                mesh: wall_mesh,
                                ..default()
                            },
                            collider,
                            RigidBody::Fixed,
                            Selectable::default(),
                            Name::new(format!("Wall {} {}", pos.x, pos.z)),
                            PlayerInteractable,
                            NavMeshAffector,
                            Minable {
                                max_health: 5.,
                                remaining_health: 5.,
                            },
                        ))
                        .id();

                    wall_entity = Some(id)
                }

                let mut floor = commands.spawn((
                    SceneBundle {
                        scene: my_assets.floor.clone(),
                        transform: Transform::from_translation(pos - Vec3::Y),
                        ..default()
                    },
                    Name::new(format!("Floor {} {}", pos.x, pos.z)),
                ));

                if level.within(x, z) {
                    floor.insert((
                        Standable,
                        PlayerInteractable,
                        Selectable {
                            selection_ring_offset: Vec3::Y * 3.0,
                        },
                        NavMeshAffector,
                        Collider::cuboid(TILE_SIZE / 2.0, 1.0, TILE_SIZE / 2.0),
                        RigidBody::Fixed,
                    ));
                }

                world_tile_tracker.tiles.insert(
                    world_tile_position,
                    WorldTile {
                        wall_entity,
                        floor_entity: floor.id(),
                    },
                );
            }
        }
    }

    // let to_update = updated_positions.iter().flat_map(|p| p.neighbors()).unique();
    //
    // for pos in to_update {
    //
    // }
}

fn get_wall_mesh(
    level: &GameLevel,
    position: &GridPosition,
    my_assets: &MyAssets,
) -> Option<(Handle<Mesh>, Quat)> {
    if level.is_open(position.x, position.z) {
        return None;
    }

    let has_north_neighbor = !level.is_open(position.x, position.z + 1);
    let has_south_neighbor = !level.is_open(position.x, position.z - 1);
    let has_east_neighbor = !level.is_open(position.x + 1, position.z);
    let has_west_neighbor = !level.is_open(position.x - 1, position.z);

    let neighbor_count = [
        has_north_neighbor,
        has_south_neighbor,
        has_east_neighbor,
        has_west_neighbor,
    ]
    .into_iter()
    .filter(|b| *b)
    .count();

    match neighbor_count {
        2 => {
            let mesh = my_assets.outer_corner_wall_mesh.clone();

            let rotation: f32 = match (
                has_north_neighbor,
                has_south_neighbor,
                has_east_neighbor,
                has_west_neighbor,
            ) {
                (true, false, true, false) => 0.,
                (true, false, false, true) => 270.,
                (false, true, false, true) => 180.,
                (false, true, true, false) => 90.,
                _ => 0.,
            };

            Some((mesh, Quat::from_rotation_y(rotation.to_radians())))
        }
        3 => {
            let mesh = my_assets.three_way_wall_mesh.clone();

            let rotation: f32 = match (
                has_north_neighbor,
                has_south_neighbor,
                has_east_neighbor,
                has_west_neighbor,
            ) {
                (true, true, true, false) => 90.,
                (true, true, false, true) => 270.,
                (true, false, true, true) => 0.,
                (false, true, true, true) => 180.,
                _ => 0.,
            };

            Some((mesh, Quat::from_rotation_y(rotation.to_radians())))
        }
        4 => {
            let north_east_open = level.is_open(position.x + 1, position.z + 1);
            let south_east_open = level.is_open(position.x + 1, position.z - 1);
            let north_west_open = level.is_open(position.x - 1, position.z + 1);
            let south_west_open = level.is_open(position.x - 1, position.z - 1);
            if north_east_open || south_east_open || north_west_open || south_west_open {
                let mesh = my_assets.inner_corner_wall_mesh.clone();

                let rotation: f32 = match (north_east_open, south_east_open, north_west_open, south_west_open) {
                    (true, false, false, false) => 180.,
                    (false, true, false, false) => 270.,
                    (false, false, true, false) => 90.,
                    _ => 0.,
                };


                Some((mesh, Quat::from_rotation_y(rotation.to_radians())))
            } else {
                let mesh = my_assets.full_wall_mesh.clone();
                Some((mesh, Quat::default()))
            }
        }
        // This should really never happen
        _ => None,
    }
}
