use crate::prelude::*;
use bevy::prelude::shape::Torus;
use bevy::render::primitives::Aabb;

#[derive(Component)]
pub struct Selected;

#[derive(Component, Default)]
pub struct Selectable {
    pub selection_ring_offset: Vec3,
}

pub enum WantToSelect {
    Additionally(Entity),
    Exclusively(Entity),
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WantToSelect>()
            .add_system(select_exclusively)
            .add_system(highlight_selected)
            .add_system(unhighlight_deselected)
            .add_startup_system(add_glow_highlight_material);
    }
}

fn select_exclusively(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    mut want_to_select: EventReader<WantToSelect>,
) {
    for event in want_to_select.iter() {
        match event {
            WantToSelect::Additionally(entity) => {
                commands.entity(*entity).insert(Selected);
            }
            WantToSelect::Exclusively(entity) => {
                for to_deselect in query.iter() {
                    if to_deselect != *entity {
                        commands.entity(to_deselect).remove::<Selected>();
                    }
                }

                commands.entity(*entity).insert(Selected);
            }
        }
    }
}

#[derive(Component)]
struct GlowHighlight;

#[derive(Resource)]
struct GlowHighlightMaterial(Handle<StandardMaterial>);

fn add_glow_highlight_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE,
        ..default()
    });

    commands.insert_resource(GlowHighlightMaterial(material_handle));
}

fn highlight_selected(
    selected: Query<(Entity, &GlobalTransform, &Selectable), Added<Selected>>,
    mut commands: Commands,
    children: Query<&Children>,
    mesh_query: Query<(&Aabb, &GlobalTransform)>,
    glow_material: Res<GlowHighlightMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, main_transform, selectable) in selected.iter() {
        let mut extreme_min = main_transform.translation_vec3a();
        let mut extreme_max = extreme_min.clone();

        for child in children.iter_descendants(entity) {
            if let Ok((aabb, transform)) = mesh_query.get(child) {
                let global_center = transform.translation_vec3a() + aabb.center;

                let min = global_center - aabb.half_extents;
                let max = global_center + aabb.half_extents;

                extreme_min = extreme_min.min(min);
                extreme_max = extreme_max.max(max);
            }
        }

        let limit = Aabb::from_min_max(extreme_min.into(), extreme_max.into());

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                GlowHighlight,
                PbrBundle {
                    mesh: meshes.add(
                        Torus {
                            radius: limit.half_extents.max_element() * 1.4,
                            ring_radius: 0.5,
                            subdivisions_segments: Torus::default().subdivisions_segments * 4,
                            subdivisions_sides: Torus::default().subdivisions_sides * 4,
                        }
                        .into(),
                    ),
                    material: glow_material.0.clone(),
                    transform: Transform::from_translation(selectable.selection_ring_offset),
                    ..default()
                },
            ));
        });
    }
}

fn unhighlight_deselected(
    mut deselected: RemovedComponents<Selected>,
    mut commands: Commands,
    parent_query: Query<&Children>,
    glowing_query: Query<&Parent, With<GlowHighlight>>,
) {
    for entity in deselected.iter() {

        if let Ok(list) = parent_query.get(entity) {
            for glow in list.iter() {
                if let Ok(parent) = glowing_query.get(*glow) {
                    commands.entity(parent.get()).remove_children(&[*glow]);
                    commands.entity(*glow).despawn_recursive();
                }
            }
        }
    }
}
