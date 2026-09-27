use godot::classes::base_material_3d::Feature;
use godot::classes::{
    Area3D, BoxMesh, BoxShape3D, CollisionShape3D, IArea3D, MeshInstance3D, StandardMaterial3D,
};
use godot::prelude::*;

use crate::player::Player;

#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct ExtractionDoor {
    base: Base<Area3D>,
    pub player: Option<Gd<Player>>,
}

#[godot_api]
impl IArea3D for ExtractionDoor {
    fn init(base: Base<Area3D>) -> Self {
        Self { base, player: None }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("extraction_door");

        let mut mesh = BoxMesh::new_gd();
        mesh.set_size(Vector3::new(1.4, 2.6, 0.3));

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.1, 0.6, 0.25, 1.0));
        mat.set_feature(Feature::EMISSION, true);
        mat.set_emission(Color::from_rgba(0.1, 0.7, 0.2, 1.0));

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, &mat);
        mesh_instance.set_position(Vector3::new(0.0, 1.3, 0.0));
        self.base_mut().add_child(&mesh_instance);

        let mut shape = BoxShape3D::new_gd();
        shape.set_size(Vector3::new(1.4, 2.6, 0.6));
        let mut collider = CollisionShape3D::new_alloc();
        collider.set_shape(&shape);
        collider.set_position(Vector3::new(0.0, 1.3, 0.0));
        self.base_mut().add_child(&collider);
    }

    fn physics_process(&mut self, _delta: f64) {
        for body in self.base().get_overlapping_bodies().iter_shared() {
            if body.is_in_group("player") {
                if let Some(mut player) = self.player.clone() {
                    player.bind_mut().reached_extraction = true;
                }
                return;
            }
        }
    }
}
