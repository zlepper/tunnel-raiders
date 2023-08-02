mod designation_gizmo;
mod menu_gizmo;
mod button_gizmo;

use crate::prelude::*;
use crate::{has_any_query_matches, GameState};
use bevy_ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use std::marker::PhantomData;
pub use designation_gizmo::*;
pub use menu_gizmo::*;
pub use button_gizmo::*;

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            add_buttons_to_gizmos
                .run_if(has_any_query_matches::<(With<RenderedGizmo>, Without<Button>)>),
        )
        .add_systems(Update, new_display_gizmos);
    }
}

const GIZMO_GAP: f32 = 5.;
const GIZMO_SIZE: f32 = 64.;
const MULTI_SPACING: f32 = GIZMO_GAP + GIZMO_SIZE;

fn add_buttons_to_gizmos(
    gizmos: Query<(Entity, &RenderedGizmo), Without<Button>>,
    mut commands: Commands,
) {
    for (entity, gizmo) in gizmos.iter() {
        let depth = gizmo.depth as f32;

        commands
            .entity(entity)
            .insert(ButtonBundle {
                style: Style {
                    top: Val::Px(GIZMO_GAP),
                    right: Val::Px(depth * MULTI_SPACING + GIZMO_GAP),
                    height: Val::Px(GIZMO_SIZE),
                    width: Val::Px(GIZMO_SIZE),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(ImageBundle {
                    image: UiImage {
                        texture: gizmo.icon.clone(),
                        ..default()
                    },
                    style: Style {
                        height: Val::Px(64.),
                        width: Val::Px(64.),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ..default()
                });
                p.spawn(
                    TextBundle::from_section(
                        &gizmo.name,
                        TextStyle {
                            font_size: 16.,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.),
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            ..default()
                        },
                        ..default()
                    }),
                );
            });
    }
}

fn new_display_gizmos(mut q: Query<(&RenderedGizmo, &mut Style)>) {

    let groups = q.iter_mut().group_by(|(g, _)| g.depth);

    for (_, group) in &groups {

        let buttons = group
            .sorted_by_key(|(g, _)| g.order)
            .map(|(_, s)| s)
            .enumerate();

        for (order, mut style) in buttons {
            let order = order as f32;
            let expected = Val::Px(order * MULTI_SPACING + GIZMO_GAP);

            if style.top != expected {
                style.top = expected;
            }
        }
    }


}

pub trait BaseGizmo {
    fn get_icon(&self) -> Handle<Image>;
    fn get_name(&self) -> String;
    fn get_order(&self) -> i32;
}

pub trait GizmoVisibility {
    type WorldQuery: WorldQuery + 'static;
    type ReadOnlyWorldQuery: ReadOnlyWorldQuery + 'static;

    fn is_visible(query: &Query<Self::WorldQuery, Self::ReadOnlyWorldQuery>) -> bool;
    fn depth() -> usize {
        0
    }
}

pub trait Gizmo: BaseGizmo + Resource + 'static {
    type Assets: Resource + 'static;

    fn initialize(assets: &Self::Assets) -> Self;
}

#[derive(Component)]
struct RenderedGizmo {
    pub icon: Handle<Image>,
    pub name: String,
    pub order: i32,
    pub depth: usize,
}

#[derive(Component)]
pub struct GizmoTag<T>(PhantomData<fn() -> T>);

impl<T> Default for GizmoTag<T> {
    fn default() -> Self {
        GizmoTag(PhantomData)
    }
}

fn maintain_gizmo<G: Gizmo + GizmoVisibility>(
    q: Query<G::WorldQuery, G::ReadOnlyWorldQuery>,
    mut commands: Commands,
    rendered_gizmo: Query<Entity, With<GizmoTag<G>>>,
    gizmo_info: Res<G>,
) {
    let should_be_visible = G::is_visible(&q);

    let is_visible = !rendered_gizmo.is_empty();

    match (should_be_visible, is_visible) {
        (true, false) => {
            commands.spawn((
                GizmoTag::<G>::default(),
                RenderedGizmo {
                    icon: gizmo_info.get_icon(),
                    name: gizmo_info.get_name(),
                    order: gizmo_info.get_order(),
                    depth: G::depth(),
                },
            ));
        }
        (false, true) => {
            for entity in rendered_gizmo.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
        _ => {}
    }
}

fn initialize_gizmo_resource<G: Gizmo>(assets: Res<G::Assets>, mut commands: Commands) {
    commands.insert_resource(G::initialize(&assets));
}

pub fn add_base_gizmo_systems<G: Gizmo + GizmoVisibility + 'static>(app: &mut App) {
    app.add_systems(Update, maintain_gizmo::<G>.run_if(resource_exists::<G>()))
        .add_systems(OnEnter(GameState::Playing), initialize_gizmo_resource::<G>);
}

