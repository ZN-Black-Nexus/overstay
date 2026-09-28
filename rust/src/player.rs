use std::collections::HashSet;

use godot::classes::base_material_3d::Feature;
use godot::classes::{
    BoxMesh, CapsuleShape3D, Camera3D, CharacterBody3D, CollisionShape3D, ICharacterBody3D,
    Input, InputEvent, InputEventMouseMotion, MeshInstance3D, Node3D, RayCast3D,
    StandardMaterial3D,
};
use godot::global::{Key, MouseButton};
use godot::prelude::*;

use crate::abomination::Abomination;
use crate::level::CELL_SIZE;
use crate::wall_creature::WallCreature;

const EYE_HEIGHT: f32 = 1.6;
const WALK_SPEED: f32 = 4.0;
const SPRINT_SPEED: f32 = 7.0;
const MOUSE_SENSITIVITY: f32 = 0.0035;
const WEAPON_RANGE: f32 = 30.0;
const FIRE_COOLDOWN: f32 = 0.35;
const STILL_FIXATION_DELAY: f32 = 1.5;
const STARE_FIXATION_DELAY: f32 = 2.0;
const MAX_AMMO: i32 = 12;

#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct Player {
    base: Base<CharacterBody3D>,
    camera: Option<Gd<Camera3D>>,
    ray: Option<Gd<RayCast3D>>,

    yaw: f32,
    pitch: f32,

    pub coins: i32,
    pub ammo: i32,
    pub is_dead: bool,
    pub reached_extraction: bool,

    /// Local, behavior-driven signal. `GameRoot` drains this each frame into the
    /// global Dread meter's trigger logic.
    pub pending_fixation: f64,

    still_timer: f32,
    stare_timer: f32,
    last_stare_target: Option<InstanceId>,
    visited: HashSet<(i32, i32)>,
    last_cell: (i32, i32),

    fire_cooldown: f32,
    knockback: Vector3,
    hallucination_timer: f32,

    gun_model: Option<Gd<Node3D>>,
    gun_rest_pos: Vector3,
    gun_kick: f32,
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            base,
            camera: None,
            ray: None,
            yaw: 0.0,
            pitch: 0.0,
            coins: 0,
            ammo: MAX_AMMO / 2,
            is_dead: false,
            reached_extraction: false,
            pending_fixation: 0.0,
            still_timer: 0.0,
            stare_timer: 0.0,
            last_stare_target: None,
            visited: HashSet::new(),
            last_cell: (0, 0),
            fire_cooldown: 0.0,
            knockback: Vector3::ZERO,
            hallucination_timer: 0.0,
            gun_model: None,
            gun_rest_pos: Vector3::ZERO,
            gun_kick: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("player");

        let mut shape = CapsuleShape3D::new_gd();
        shape.set_radius(0.35);
        shape.set_height(1.8);
        let mut collider = CollisionShape3D::new_alloc();
        collider.set_shape(&shape);
        collider.set_position(Vector3::new(0.0, 0.9, 0.0));
        self.base_mut().add_child(&collider);

        let mut camera = Camera3D::new_alloc();
        camera.set_position(Vector3::new(0.0, EYE_HEIGHT, 0.0));
        camera.set_current(true);
        self.base_mut().add_child(&camera);

        let mut ray = RayCast3D::new_alloc();
        ray.set_target_position(Vector3::new(0.0, 0.0, -WEAPON_RANGE));
        ray.set_enabled(true);
        camera.add_child(&ray);

        self.gun_rest_pos = Vector3::new(0.22, -0.22, -0.45);
        let gun_model = build_gun_model();
        camera.add_child(&gun_model);
        let mut gun_model = gun_model;
        gun_model.set_position(self.gun_rest_pos);
        self.gun_model = Some(gun_model);

        self.camera = Some(camera);
        self.ray = Some(ray);

        Input::singleton().set_mouse_mode(godot::classes::input::MouseMode::CAPTURED);
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if self.is_dead || self.reached_extraction {
            return;
        }
        if let Ok(motion) = event.try_cast::<InputEventMouseMotion>() {
            let rel = motion.get_relative();
            self.yaw -= rel.x * MOUSE_SENSITIVITY;
            self.pitch = (self.pitch - rel.y * MOUSE_SENSITIVITY).clamp(-1.4, 1.4);

            let yaw = self.yaw;
            let pitch = self.pitch;
            self.base_mut().set_rotation(Vector3::new(0.0, yaw, 0.0));
            if let Some(camera) = self.camera.as_mut() {
                camera.set_rotation(Vector3::new(pitch, 0.0, 0.0));
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.is_dead || self.reached_extraction {
            self.base_mut().set_velocity(Vector3::ZERO);
            self.base_mut().move_and_slide();
            return;
        }

        let dt = delta as f32;
        self.handle_movement(dt);
        self.handle_weapon(dt);
        self.update_fixation(dt);
        self.update_gun_kick(dt);

        if self.hallucination_timer > 0.0 {
            self.hallucination_timer -= dt;
        }
    }
}

impl Player {
    fn handle_movement(&mut self, dt: f32) {
        let input = Input::singleton();
        let mut input_dir = Vector2::ZERO;
        if input.is_key_pressed(Key::W) {
            input_dir.y += 1.0;
        }
        if input.is_key_pressed(Key::S) {
            input_dir.y -= 1.0;
        }
        if input.is_key_pressed(Key::D) {
            input_dir.x += 1.0;
        }
        if input.is_key_pressed(Key::A) {
            input_dir.x -= 1.0;
        }
        let input_dir = input_dir.normalized_or_zero();

        let yaw = self.yaw;
        let forward = Vector3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vector3::new(yaw.cos(), 0.0, -yaw.sin());

        let sprinting = input.is_key_pressed(Key::SHIFT);
        let speed = if sprinting { SPRINT_SPEED } else { WALK_SPEED };

        let world_dir = forward * input_dir.y + right * input_dir.x;

        let mut velocity = self.base().get_velocity();
        velocity.x = world_dir.x * speed + self.knockback.x;
        velocity.z = world_dir.z * speed + self.knockback.z;
        velocity.y = if self.base().is_on_floor() { -0.1 } else { velocity.y - 9.8 * dt };

        self.knockback = self.knockback.lerp(Vector3::ZERO, (dt * 4.0).min(1.0));

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }

    fn handle_weapon(&mut self, dt: f32) {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);

        let firing = Input::singleton().is_mouse_button_pressed(MouseButton::LEFT);
        if !firing || self.fire_cooldown > 0.0 || self.ammo <= 0 {
            return;
        }

        self.fire_cooldown = FIRE_COOLDOWN;
        self.ammo -= 1;
        self.gun_kick = 1.0;

        let Some(ray) = self.ray.clone() else { return };
        if !ray.is_colliding() {
            return;
        }
        let Some(collider) = ray.get_collider() else { return };

        if let Ok(mut creature) = collider.clone().try_cast::<WallCreature>() {
            creature.bind_mut().take_damage(1);
        } else if let Ok(mut abomination) = collider.try_cast::<Abomination>() {
            abomination.bind_mut().stagger();
        }
    }

    fn update_fixation(&mut self, dt: f32) {
        let velocity = self.base().get_velocity();
        let horiz_speed = Vector2::new(velocity.x, velocity.z).length();

        if horiz_speed < 0.15 {
            self.still_timer += dt;
        } else {
            self.still_timer = 0.0;
        }
        if self.still_timer > STILL_FIXATION_DELAY {
            self.pending_fixation += 3.0 * dt as f64;
        }

        if let Some(ray) = self.ray.clone() {
            if ray.is_colliding() {
                if let Some(collider) = ray.get_collider() {
                    let id = collider.instance_id();
                    if self.last_stare_target == Some(id) {
                        self.stare_timer += dt;
                    } else {
                        self.last_stare_target = Some(id);
                        self.stare_timer = 0.0;
                    }
                }
            } else {
                self.last_stare_target = None;
                self.stare_timer = 0.0;
            }
        }
        if self.stare_timer > STARE_FIXATION_DELAY {
            self.pending_fixation += 2.0 * dt as f64;
        }

        let pos = self.base().get_global_position();
        let cell = (
            (pos.x / CELL_SIZE).round() as i32,
            (pos.z / CELL_SIZE).round() as i32,
        );
        if cell != self.last_cell {
            if self.visited.contains(&cell) {
                self.pending_fixation += 4.0;
            } else {
                self.visited.insert(cell);
                self.pending_fixation = (self.pending_fixation - 2.0).max(0.0);
            }
            self.last_cell = cell;
        }
    }

    pub fn global_pos(&self) -> Vector3 {
        self.base().get_global_position()
    }

    pub fn add_coins(&mut self, amount: i32) {
        self.coins += amount;
    }

    pub fn add_ammo(&mut self, amount: i32) {
        self.ammo = (self.ammo + amount).min(MAX_AMMO);
    }

    pub fn trigger_hallucination(&mut self) {
        self.pending_fixation += 25.0;
        self.hallucination_timer = 2.5;
    }

    pub fn is_hallucinating(&self) -> bool {
        self.hallucination_timer > 0.0
    }

    pub fn apply_knockback(&mut self, from: Vector3, force: f32) {
        let dir = (self.global_pos() - from).normalized_or_zero();
        self.knockback = dir * force;
        self.pending_fixation += 6.0;
    }

    pub fn kill(&mut self) {
        self.is_dead = true;
    }

    fn update_gun_kick(&mut self, dt: f32) {
        if self.gun_kick <= 0.0 {
            return;
        }
        self.gun_kick = (self.gun_kick - dt * 6.0).max(0.0);
        if let Some(gun) = self.gun_model.as_mut() {
            let kicked = self.gun_rest_pos + Vector3::new(0.0, 0.03, 0.1) * self.gun_kick;
            gun.set_position(kicked);
        }
    }
}

/// A crude primitive-built pistol silhouette (a grip box + a barrel box) worn
/// in first person. No rigging or external model needed — same "build it from
/// simple shapes in code" approach as the rest of the game's visuals.
fn build_gun_model() -> Gd<Node3D> {
    let mut root = Node3D::new_alloc();

    let mut mat = StandardMaterial3D::new_gd();
    mat.set_albedo(Color::from_rgba(0.08, 0.08, 0.09, 1.0));
    mat.set_feature(Feature::EMISSION, true);
    mat.set_emission(Color::from_rgba(0.05, 0.05, 0.06, 1.0));

    let mut barrel_mesh = BoxMesh::new_gd();
    barrel_mesh.set_size(Vector3::new(0.06, 0.06, 0.26));
    let mut barrel = MeshInstance3D::new_alloc();
    barrel.set_mesh(&barrel_mesh);
    barrel.set_surface_override_material(0, &mat);
    barrel.set_position(Vector3::new(0.0, 0.03, -0.08));
    root.add_child(&barrel);

    let mut grip_mesh = BoxMesh::new_gd();
    grip_mesh.set_size(Vector3::new(0.055, 0.16, 0.055));
    let mut grip = MeshInstance3D::new_alloc();
    grip.set_mesh(&grip_mesh);
    grip.set_surface_override_material(0, &mat);
    grip.set_position(Vector3::new(0.0, -0.08, 0.04));
    grip.set_rotation(Vector3::new(0.35, 0.0, 0.0));
    root.add_child(&grip);

    root
}
