use crate::errands::Designation;
use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(display_gizmos.run_if(should_update_gizmo_display))
            .add_startup_system(spawn_gizmo_tracker)
            .add_system(handle_gizmo_click);
    }
}

#[derive(Debug, Eq, Clone)]
pub struct GizmoInfo {
    pub icon: Handle<Image>,
    pub name: String,
    pub order: i32,
    pub id: String,
}

impl PartialEq for GizmoInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for GizmoInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Component)]
struct GizmoButton {
    id: String,
    entities: Vec<Entity>,
}

#[derive(Component, Default)]
struct GizmoTracker {
    gizmo_buttons: HashMap<String, Entity>,
}

fn should_update_gizmo_display(
    new_selection: Query<(), (Added<Selected>, With<EntityGizmos>)>,
    removed_selection: RemovedComponents<Selected>,
    gizmos_changed: Query<(), (With<Selected>, Changed<EntityGizmos>)>,
) -> bool {
    !new_selection.is_empty() || !removed_selection.is_empty() || !gizmos_changed.is_empty()
}

fn spawn_gizmo_tracker(mut commands: Commands) {
    commands.spawn((
        GizmoTracker::default(),
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::WrapReverse,
                overflow: Overflow::Hidden,
                position: UiRect {
                    right: Val::Px(20.),
                    left: Val::Auto,
                    ..default()
                },
                size: Size::new(Val::Px(50.), Val::Auto),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
    ));
}

fn display_gizmos(
    q: Query<(&EntityGizmos, Entity), With<Selected>>,
    mut buttons: Query<(&mut GizmoButton, Entity)>,
    mut commands: Commands,
    mut gizmo_nodes: Query<(&mut GizmoTracker, Entity)>,
) {
    let Ok((mut gizmo_button_tracker, tracker_entity)) = gizmo_nodes.get_single_mut() else {
        error!("Failed to get gizmo tracker node");
        return;
    };

    let gizmos_to_render = q
        .iter()
        .flat_map(|(gizmos, entity)| gizmos.gizmos.iter().map(move |g| (g, entity)))
        .into_group_map();

    let render_order = gizmos_to_render.iter().sorted_by_key(|(g, _)| g.order);

    for (gizmo, entities) in render_order {
        let found = if let Some(existing) = gizmo_button_tracker.gizmo_buttons.get(&gizmo.id) {
            if let Ok((ref mut button, _)) = &mut buttons.get_mut(*existing) {
                button.entities = entities.clone();
                true
            } else {
                false
            }
        } else {
            false
        };

        if !found {
            let button = GizmoButton {
                id: gizmo.id.to_string(),
                entities: entities.clone(),
            };
            let entity = commands
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        ..Default::default()
                    },
                    button,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        image: UiImage {
                            texture: gizmo.icon.clone(),
                            ..default()
                        },
                        ..default()
                    });
                })
                .set_parent(tracker_entity)
                .id();

            gizmo_button_tracker
                .gizmo_buttons
                .insert(gizmo.id.clone(), entity);
        }
    }

    let rendered_gizmo_ids: HashSet<_> = gizmos_to_render.keys().map(|g| &g.id).collect();

    for (gizmo, entity) in buttons.iter() {
        if !rendered_gizmo_ids.contains(&gizmo.id) {
            commands.entity(entity).despawn_recursive();
            gizmo_button_tracker.gizmo_buttons.remove(&gizmo.id);
        }
    }
}

#[derive(Debug)]
pub enum GizmoSelectedAction {
    SetDesignation(Designation),
}

#[derive(Debug)]
pub struct GizmoContainer {
    icon: Handle<Image>,
    name: String,
    order: i32,
    id: String,
    activate: bool,
    on_selected: GizmoSelectedAction,
}

impl PartialEq for GizmoContainer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GizmoContainer {}

impl Hash for GizmoContainer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl GizmoContainer {
    pub fn new(
        id: &str,
        name: &str,
        order: i32,
        icon: Handle<Image>,
        on_selected: GizmoSelectedAction,
    ) -> Self {
        Self {
            on_selected,
            activate: false,
            order,
            id: id.to_string(),
            name: name.to_string(),
            icon,
        }
    }
}

#[derive(Component, Default)]
pub struct EntityGizmos {
    pub gizmos: Vec<GizmoContainer>,
}

fn handle_gizmo_click(
    buttons: Query<(&GizmoButton, &Interaction), Changed<Interaction>>,
    mut gizmo_entities: Query<&mut EntityGizmos, With<Selected>>,
) {
    for (gizmo, interaction) in buttons.iter() {
        if *interaction == Interaction::Clicked {
            for entity in &gizmo.entities {
                if let Ok(ref mut g) = &mut gizmo_entities.get_mut(*entity) {
                    if let Some(ref mut g) = &mut g.gizmos.iter_mut().find(|g| g.id == gizmo.id) {
                        g.activate = true;
                    }
                }
            }
        }
    }
}
