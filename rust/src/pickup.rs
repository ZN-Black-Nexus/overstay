use godot::classes::base_material_3d::Feature;
use godot::classes::{
    Area3D, BoxMesh, CollisionShape3D, CylinderMesh, IArea3D, MeshInstance3D, SphereShape3D,
    StandardMaterial3D,
};
use godot::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::player::Player;

const FAKE_COIN_CHANCE: f64 = 0.12;
const COIN_SPIN_SPEED: f32 = 2.4;

/// Coins are personal loot, not shared. Visually a fake coin is indistinguishable
/// from a real one — picking it up triggers a harmless-but-scary hallucination
/// instead of paying out, deliberately muddying the same tells used elsewhere to
/// spot trouble.
#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct Coin {
    base: Base<Area3D>,
    pub player: Option<Gd<Player>>,
    is_fake: bool,
}

#[godot_api]
impl IArea3D for Coin {
    fn init(base: Base<Area3D>) -> Self {
        Self {
            base,
            player: None,
            is_fake: rand::random_bool(FAKE_COIN_CHANCE),
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("coin");

        // A flat disc standing on edge (tilted 90°), not a ball — a real coin
        // shape, spun around the vertical axis each frame for the classic
        // coin-pickup flicker.
        let mut mesh = CylinderMesh::new_gd();
        mesh.set_top_radius(0.22);
        mesh.set_bottom_radius(0.22);
        mesh.set_height(0.045);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.85, 0.7, 0.15, 1.0));
        mat.set_feature(Feature::EMISSION, true);
        mat.set_emission(Color::from_rgba(0.6, 0.45, 0.05, 1.0));

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, &mat);
        mesh_instance.set_rotation(Vector3::new(0.0, 0.0, FRAC_PI_2));
        self.base_mut().add_child(&mesh_instance);

        let mut shape = SphereShape3D::new_gd();
        shape.set_radius(0.5);
        let mut collider = CollisionShape3D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);
    }

    fn physics_process(&mut self, delta: f64) {
        self.base_mut().rotate_y(COIN_SPIN_SPEED * delta as f32);

        for body in self.base().get_overlapping_bodies().iter_shared() {
            if body.is_in_group("player") {
                if let Some(mut player) = self.player.clone() {
                    if self.is_fake {
                        player.bind_mut().trigger_hallucination();
                    } else {
                        player.bind_mut().add_coins(1);
                    }
                }
                self.base_mut().queue_free();
                return;
            }
        }
    }
}

#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct AmmoPickup {
    base: Base<Area3D>,
    pub player: Option<Gd<Player>>,
}

#[godot_api]
impl IArea3D for AmmoPickup {
    fn init(base: Base<Area3D>) -> Self {
        Self { base, player: None }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("ammo_pickup");

        // A squat case shape reads more clearly as a pickup than a tall thin
        // pillar (which looked cylinder-ish at a glance).
        let mut mesh = BoxMesh::new_gd();
        mesh.set_size(Vector3::new(0.26, 0.16, 0.18));

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.3, 0.3, 0.32, 1.0));
        mat.set_feature(Feature::EMISSION, true);
        mat.set_emission(Color::from_rgba(0.15, 0.25, 0.35, 1.0));

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, &mat);
        self.base_mut().add_child(&mesh_instance);

        let mut shape = SphereShape3D::new_gd();
        shape.set_radius(0.5);
        let mut collider = CollisionShape3D::new_alloc();
        collider.set_shape(&shape);
        self.base_mut().add_child(&collider);
    }

    fn physics_process(&mut self, _delta: f64) {
        for body in self.base().get_overlapping_bodies().iter_shared() {
            if body.is_in_group("player") {
                if let Some(mut player) = self.player.clone() {
                    player.bind_mut().add_ammo(6);
                }
                self.base_mut().queue_free();
                return;
            }
        }
    }
}
