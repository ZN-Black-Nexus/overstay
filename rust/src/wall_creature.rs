use godot::classes::base_material_3d::Feature;
use godot::classes::{CapsuleMesh, INode3D, MeshInstance3D, Node3D, StandardMaterial3D};
use godot::prelude::*;

use crate::player::Player;

const SPEED: f32 = 3.4;
const LIFETIME: f32 = 8.0;
const CONTACT_RADIUS: f32 = 1.1;
const CONTACT_COOLDOWN: f32 = 1.5;
const KNOCKBACK_FORCE: f32 = 5.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Active,
    Killed,
    TimedOut,
}

/// A temporary, corporeal threat that tears free of a wall/floor/ceiling once it has
/// absorbed enough Fixation. Chases the player briefly, then either gets killed
/// (small Dread cost) or times out unresolved (bigger Dread cost).
#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct WallCreature {
    base: Base<Node3D>,
    pub player: Option<Gd<Player>>,
    health: i32,
    lifetime: f32,
    contact_cooldown: f32,
    pub resolution: Resolution,
}

#[godot_api]
impl INode3D for WallCreature {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player: None,
            health: 2,
            lifetime: LIFETIME,
            contact_cooldown: 0.0,
            resolution: Resolution::Active,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("wall_creature");

        let mut mesh = CapsuleMesh::new_gd();
        mesh.set_radius(0.4);
        mesh.set_height(1.9);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.04, 0.04, 0.04, 1.0));
        mat.set_feature(Feature::EMISSION, true);
        mat.set_emission(Color::from_rgba(0.5, 0.02, 0.02, 1.0));

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, &mat);
        mesh_instance.set_position(Vector3::new(0.0, 0.95, 0.0));
        self.base_mut().add_child(&mesh_instance);
    }

    fn physics_process(&mut self, delta: f64) {
        if self.resolution != Resolution::Active {
            return;
        }

        let dt = delta as f32;
        self.lifetime -= dt;
        self.contact_cooldown = (self.contact_cooldown - dt).max(0.0);

        if self.lifetime <= 0.0 {
            self.resolution = Resolution::TimedOut;
            return;
        }

        let Some(player) = self.player.clone() else { return };
        let target = player.bind().global_pos();
        let my_pos = self.base().get_global_position();
        let to_target = target - my_pos;
        let dist = to_target.length();

        if dist > 0.05 {
            let step = to_target.normalized() * SPEED * dt;
            self.base_mut().set_global_position(my_pos + step);
        }

        if dist < CONTACT_RADIUS && self.contact_cooldown <= 0.0 {
            self.contact_cooldown = CONTACT_COOLDOWN;
            let mut player = player;
            player.bind_mut().apply_knockback(my_pos, KNOCKBACK_FORCE);
        }
    }
}

impl WallCreature {
    pub fn take_damage(&mut self, dmg: i32) {
        if self.resolution != Resolution::Active {
            return;
        }
        self.health -= dmg;
        if self.health <= 0 {
            self.resolution = Resolution::Killed;
        }
    }
}
