use crate::gizmos::BaseGizmo;
use crate::prelude::{Handle, Image};

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
