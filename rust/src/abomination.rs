use godot::classes::base_material_3d::Feature;
use godot::classes::{CapsuleMesh, INode3D, MeshInstance3D, Node3D, StandardMaterial3D};
use godot::prelude::*;

use crate::player::Player;

const SPEED: f32 = 4.6;
const KILL_RADIUS: f32 = 1.2;
const STAGGER_DURATION: f32 = 3.0;

/// The rare, catastrophic escalation: unstoppable, cannot be killed (only staggered
/// with the sidearm), and kills the player on contact.
#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct Abomination {
    base: Base<Node3D>,
    pub player: Option<Gd<Player>>,
    stagger_timer: f32,
}

#[godot_api]
impl INode3D for Abomination {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player: None,
            stagger_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("abomination");

        let mut mesh = CapsuleMesh::new_gd();
        mesh.set_radius(0.55);
        mesh.set_height(2.6);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.01, 0.01, 0.01, 1.0));
        mat.set_feature(Feature::EMISSION, true);
        mat.set_emission(Color::from_rgba(0.3, 0.0, 0.0, 1.0));

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, &mat);
        mesh_instance.set_position(Vector3::new(0.0, 1.3, 0.0));
        self.base_mut().add_child(&mesh_instance);
    }

    fn physics_process(&mut self, delta: f64) {
        let dt = delta as f32;

        if self.stagger_timer > 0.0 {
            self.stagger_timer -= dt;
            return;
        }

        let Some(player) = self.player.clone() else { return };
        if player.bind().is_dead {
            return;
        }

        let target = player.bind().global_pos();
        let my_pos = self.base().get_global_position();
        let to_target = target - my_pos;
        let dist = to_target.length();

        if dist > 0.05 {
            let step = to_target.normalized() * SPEED * dt;
            self.base_mut().set_global_position(my_pos + step);
        }

        if dist < KILL_RADIUS {
            let mut player = player;
            player.bind_mut().kill();
        }
    }
}

impl Abomination {
    pub fn stagger(&mut self) {
        self.stagger_timer = STAGGER_DURATION;
    }
}
