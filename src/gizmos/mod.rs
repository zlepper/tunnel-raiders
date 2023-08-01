use crate::errands::Designation;
use crate::prelude::*;
use crate::{has_any_query_matches, GameState};
use bevy_ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use std::marker::PhantomData;

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

fn add_buttons_to_gizmos(
    gizmos: Query<(Entity, &RenderedGizmo), Without<Button>>,
    mut commands: Commands,
) {
    for (entity, gizmo) in gizmos.iter() {
        commands
            .entity(entity)
            .insert(ButtonBundle {
                style: Style {
                    top: Val::Px(GIZMO_GAP),
                    right: Val::Px(GIZMO_GAP),
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
                p.spawn(TextBundle::from_section(
                    &gizmo.name,
                    TextStyle {
                        font_size: 16.,
                        color: Color::WHITE,
                        ..default()
                    },
                ).with_style(Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.),
                    margin: UiRect  {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                }));
            });
    }
}

fn new_display_gizmos(mut q: Query<(&RenderedGizmo, &mut Style)>) {
    let buttons = q
        .iter_mut()
        .sorted_by_key(|(g, _)| g.order)
        .map(|(_, s)| s)
        .enumerate();

    for (order, mut style) in buttons {
        let order = order as f32;
        let expected = Val::Px(order * GIZMO_SIZE + (order + 1.) * GIZMO_GAP);

        if style.top != expected {
            style.top = expected;
        }
    }
}

pub trait BaseGizmo {
    fn get_icon(&self) -> Handle<Image>;
    fn get_name(&self) -> String;
    fn get_order(&self) -> i32;
}

pub struct ButtonGizmo {
    pub icon: Handle<Image>,
    pub name: String,
    pub order: i32,
}

impl ButtonGizmo {
    pub fn new(icon: Handle<Image>, name: &str, order: i32) -> Self {
        Self {
            icon,
            name: name.to_string(),
            order,
        }
    }
}

pub trait HasBaseGizmo {
    fn get_base_gizmo(&self) -> &ButtonGizmo;
}

impl<G> BaseGizmo for G
where
    G: HasBaseGizmo,
{
    fn get_icon(&self) -> Handle<Image> {
        self.get_base_gizmo().get_icon()
    }

    fn get_name(&self) -> String {
        self.get_base_gizmo().get_name()
    }

    fn get_order(&self) -> i32 {
        self.get_base_gizmo().get_order()
    }
}

impl BaseGizmo for ButtonGizmo {
    fn get_icon(&self) -> Handle<Image> {
        self.icon.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_order(&self) -> i32 {
        self.order
    }
}

pub trait ApplicableGizmo {
    type WorldQuery: WorldQuery + 'static;
    type ReadOnlyWorldQuery: ReadOnlyWorldQuery + 'static;

    fn apply(
        commands: &mut Commands,
        query: &Query<(Entity, Self::WorldQuery), Self::ReadOnlyWorldQuery>,
    );
}

pub trait Gizmo: ApplicableGizmo + BaseGizmo + Resource + 'static {
    type Assets: Resource + 'static;

    fn is_visible(query: &Query<Self::WorldQuery, Self::ReadOnlyWorldQuery>) -> bool;
    fn initialize(assets: &Self::Assets) -> Self;
}

pub trait DesignationGizmo {
    type WorldQuery: WorldQuery + 'static;
    type ReadOnlyWorldQuery: ReadOnlyWorldQuery + 'static;
    type Errand: Errand + 'static;

    fn create_errand(entity: Entity) -> Self::Errand;
}

impl<G: DesignationGizmo> ApplicableGizmo for G {
    type WorldQuery = G::WorldQuery;
    type ReadOnlyWorldQuery = G::ReadOnlyWorldQuery;

    fn apply(
        commands: &mut Commands,
        query: &Query<(Entity, Self::WorldQuery), Self::ReadOnlyWorldQuery>,
    ) {
        for (entity, _) in query.iter() {
            let errand = G::create_errand(entity);
            commands
                .entity(entity)
                .insert(Designation::new(entity, errand));
        }
    }
}

#[derive(Component)]
struct RenderedGizmo {
    pub icon: Handle<Image>,
    pub name: String,
    pub order: i32,
}

#[derive(Component)]
struct GizmoTag<T>(PhantomData<fn() -> T>);

impl<T> Default for GizmoTag<T> {
    fn default() -> Self {
        GizmoTag(PhantomData)
    }
}

fn maintain_gizmo<G: Gizmo>(
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

fn activate_gizmo_on_click<G: Gizmo>(
    activated: Query<&Interaction, (With<GizmoTag<G>>, Changed<Interaction>)>,
    mut commands: Commands,
    gizmo_query: Query<(Entity, G::WorldQuery), G::ReadOnlyWorldQuery>,
) {
    for interaction in activated.iter() {
        if *interaction == Interaction::Pressed {
            G::apply(&mut commands, &gizmo_query);
        }
    }
}

pub trait GizmoAppExtension {
    fn add_gizmo<G: Gizmo + 'static>(&mut self) -> &mut Self;
}

impl GizmoAppExtension for App {
    fn add_gizmo<G: Gizmo + 'static>(&mut self) -> &mut Self {
        self.add_systems(Update, maintain_gizmo::<G>.run_if(resource_exists::<G>()))
            .add_systems(OnEnter(GameState::Playing), initialize_gizmo_resource::<G>)
            .add_systems(Update, activate_gizmo_on_click::<G>)
    }
}
