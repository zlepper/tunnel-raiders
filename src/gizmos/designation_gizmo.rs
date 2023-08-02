use crate::errands::Designation;
use crate::gizmos::{add_base_gizmo_systems, GizmoTag, GizmoVisibility};
use crate::prelude::*;

pub trait DesignationGizmo: Gizmo + GizmoVisibility {
    type Errand: Errand + 'static;

    fn create_errand(entity: Entity) -> Self::Errand;
}


fn apply_designation_gizmo<G: DesignationGizmo>(
    activated: Query<&Interaction, (With<GizmoTag<G>>, Changed<Interaction>)>,
    mut commands: Commands,
    gizmo_query: Query<(Entity, G::WorldQuery), G::ReadOnlyWorldQuery>,
) {
    let activate = activated.iter().any(|i| *i == Interaction::Pressed);

    if !activate {
        return;
    }

    for (entity, _) in gizmo_query.iter() {
        let errand = G::create_errand(entity);
        commands
            .entity(entity)
            .insert(Designation::new(entity, errand));
    }
}


pub trait DesignationGizmoAppExtension {
    fn add_designation_gizmo<G: DesignationGizmo + 'static>(&mut self) -> &mut Self;
}

impl DesignationGizmoAppExtension for App {
    fn add_designation_gizmo<G: DesignationGizmo + 'static>(&mut self) -> &mut Self {
        add_base_gizmo_systems::<G>(self);
        self.add_systems(Update, apply_designation_gizmo::<G>)
    }
}
