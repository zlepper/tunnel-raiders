use crate::gizmos::{add_base_gizmo_systems, GizmoTag, GizmoVisibility};
use crate::prelude::*;

pub trait MenuGizmo: Gizmo + GizmoVisibility {}

#[derive(Component, Eq, PartialEq, Clone, Debug)]
pub enum MenuGizmoState {
    Open,
    Closed,
}

impl MenuGizmoState {
    fn toggle(&mut self) {
        *self = match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

fn toggle_menu_gizmo<G: MenuGizmo>(
    mut activated: Query<
        (&Interaction, &mut MenuGizmoState),
        (With<GizmoTag<G>>, Changed<Interaction>),
    >,
) {
    for (interaction, mut state) in activated.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.toggle();
        }
    }
}

fn add_menu_state_to_menu_gizmo<G: MenuGizmo>(q: Query<Entity, Added<GizmoTag<G>>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).insert(MenuGizmoState::Closed);
    }
}

pub trait MenuGizmoAppExtension {
    fn add_menu_gizmo<G: MenuGizmo + 'static>(&mut self) -> &mut Self;
}

impl MenuGizmoAppExtension for App {
    fn add_menu_gizmo<G: MenuGizmo + 'static>(&mut self) -> &mut Self {
        add_base_gizmo_systems::<G>(self);
        self.add_systems(Update, toggle_menu_gizmo::<G>)
            .add_systems(Update, add_menu_state_to_menu_gizmo::<G>)
    }
}

pub trait GizmoChildOf {
    type Parent: MenuGizmo;
}

impl<Child: GizmoChildOf> GizmoVisibility for Child
{
    type WorldQuery = &'static MenuGizmoState;
    type ReadOnlyWorldQuery = With<GizmoTag<Child::Parent>>;

    fn is_visible(query: &Query<Self::WorldQuery, Self::ReadOnlyWorldQuery>) -> bool {
        query.iter().any(|s| *s == MenuGizmoState::Open)
    }

    fn depth() -> usize {
        Child::Parent::depth() + 1
    }
}
