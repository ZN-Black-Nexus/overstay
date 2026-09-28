use godot::classes::base_material_3d::{BillboardMode, Feature, TextureParam};
use godot::classes::{
    Area3D, BoxMesh, BoxShape3D, CanvasLayer, CollisionShape3D, INode3D, Label, Label3D,
    MeshInstance3D, Node3D, StandardMaterial3D, StaticBody3D,
};
use godot::prelude::*;

use crate::player::Player;
use crate::texgen::generate_texture;

const ROOM_WIDTH: f32 = 10.0;
const ROOM_DEPTH: f32 = 8.0;
const WALL_HEIGHT: f32 = 3.0;

/// The pre-game hub: a small walkable room (placeholder box furniture —
/// nothing here depends on real art) where you pick Solo or Multiplayer.
/// Multiplayer is intentionally unimplemented for now.
#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct Lobby {
    base: Base<Node3D>,
    player: Option<Gd<Player>>,
    solo_terminal: Option<Gd<Area3D>>,
    mp_terminal: Option<Gd<Area3D>>,
    message_label: Option<Gd<Label>>,
}

#[godot_api]
impl INode3D for Lobby {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player: None,
            solo_terminal: None,
            mp_terminal: None,
            message_label: None,
        }
    }

    fn ready(&mut self) {
        self.build_room();
        self.build_furniture();

        let mut player = Player::new_alloc();
        self.base_mut().add_child(&player);
        player.set_global_position(Vector3::new(0.0, 0.1, ROOM_DEPTH * 0.5 - 1.5));
        self.player = Some(player);

        let solo = self.build_terminal(
            Vector3::new(-2.5, 0.0, -ROOM_DEPTH * 0.5 + 1.2),
            "SOLO",
            Color::from_rgba(0.15, 0.85, 0.3, 1.0),
        );
        self.solo_terminal = Some(solo);

        let mp = self.build_terminal(
            Vector3::new(2.5, 0.0, -ROOM_DEPTH * 0.5 + 1.2),
            "MULTIPLAYER\n(LOCKED)",
            Color::from_rgba(0.6, 0.15, 0.15, 1.0),
        );
        self.mp_terminal = Some(mp);

        self.build_hud();
    }

    fn process(&mut self, _delta: f64) {
        if let Some(term) = self.solo_terminal.clone() {
            if overlaps_player(&term) {
                let _ = self
                    .base()
                    .get_tree()
                    .change_scene_to_file("res://game.tscn");
                return;
            }
        }

        let mp_message = self
            .mp_terminal
            .clone()
            .map(|t| overlaps_player(&t))
            .unwrap_or(false);

        if let Some(l) = self.message_label.as_mut() {
            l.set_text(if mp_message {
                "Multiplayer is coming soon."
            } else {
                ""
            });
        }
    }
}

fn overlaps_player(area: &Gd<Area3D>) -> bool {
    for body in area.get_overlapping_bodies().iter_shared() {
        if body.is_in_group("player") {
            return true;
        }
    }
    false
}

impl Lobby {
    fn build_room(&mut self) {
        let wall_tex = generate_texture(
            96,
            Color::from_rgba(0.52, 0.55, 0.60, 1.0),
            Color::from_rgba(0.38, 0.41, 0.46, 1.0),
            3,
            None,
            None,
        );
        let mut wall_mat = StandardMaterial3D::new_gd();
        wall_mat.set_albedo(Color::from_rgba(0.52, 0.55, 0.60, 1.0));
        if let Some(t) = wall_tex {
            wall_mat.set_texture(TextureParam::ALBEDO, &t);
        }

        let mut floor_mat = StandardMaterial3D::new_gd();
        floor_mat.set_albedo(Color::from_rgba(0.22, 0.22, 0.24, 1.0));

        let mut ceiling_mat = StandardMaterial3D::new_gd();
        ceiling_mat.set_albedo(Color::from_rgba(0.68, 0.68, 0.70, 1.0));

        self.add_block(
            Vector3::new(0.0, -0.05, 0.0),
            Vector3::new(ROOM_WIDTH, 0.1, ROOM_DEPTH),
            &floor_mat,
            true,
        );
        self.add_block(
            Vector3::new(0.0, WALL_HEIGHT, 0.0),
            Vector3::new(ROOM_WIDTH, 0.1, ROOM_DEPTH),
            &ceiling_mat,
            false,
        );

        // Four walls, a gap-free box shell (simplest correct option for a
        // single small room — no maze connectivity to worry about here).
        self.add_block(
            Vector3::new(0.0, WALL_HEIGHT * 0.5, -ROOM_DEPTH * 0.5),
            Vector3::new(ROOM_WIDTH, WALL_HEIGHT, 0.2),
            &wall_mat,
            true,
        );
        self.add_block(
            Vector3::new(0.0, WALL_HEIGHT * 0.5, ROOM_DEPTH * 0.5),
            Vector3::new(ROOM_WIDTH, WALL_HEIGHT, 0.2),
            &wall_mat,
            true,
        );
        self.add_block(
            Vector3::new(-ROOM_WIDTH * 0.5, WALL_HEIGHT * 0.5, 0.0),
            Vector3::new(0.2, WALL_HEIGHT, ROOM_DEPTH),
            &wall_mat,
            true,
        );
        self.add_block(
            Vector3::new(ROOM_WIDTH * 0.5, WALL_HEIGHT * 0.5, 0.0),
            Vector3::new(0.2, WALL_HEIGHT, ROOM_DEPTH),
            &wall_mat,
            true,
        );
    }

    fn build_furniture(&mut self) {
        let mut couch_mat = StandardMaterial3D::new_gd();
        couch_mat.set_albedo(Color::from_rgba(0.45, 0.12, 0.14, 1.0));
        self.add_block(
            Vector3::new(-ROOM_WIDTH * 0.5 + 0.7, 0.4, 1.0),
            Vector3::new(0.7, 0.8, 2.4),
            &couch_mat,
            true,
        );

        let mut table_mat = StandardMaterial3D::new_gd();
        table_mat.set_albedo(Color::from_rgba(0.35, 0.24, 0.14, 1.0));
        self.add_block(
            Vector3::new(0.0, 0.25, 1.5),
            Vector3::new(1.4, 0.5, 0.8),
            &table_mat,
            true,
        );

        let mut shelf_mat = StandardMaterial3D::new_gd();
        shelf_mat.set_albedo(Color::from_rgba(0.28, 0.20, 0.13, 1.0));
        self.add_block(
            Vector3::new(ROOM_WIDTH * 0.5 - 0.35, 1.1, -1.5),
            Vector3::new(0.5, 2.2, 1.6),
            &shelf_mat,
            true,
        );
    }

    fn add_block(&mut self, center: Vector3, size: Vector3, material: &Gd<StandardMaterial3D>, solid: bool) {
        let mut mesh = BoxMesh::new_gd();
        mesh.set_size(size);

        let mut mesh_instance = MeshInstance3D::new_alloc();
        mesh_instance.set_mesh(&mesh);
        mesh_instance.set_surface_override_material(0, material);

        if solid {
            let mut body = StaticBody3D::new_alloc();
            body.set_position(center);

            let mut shape = BoxShape3D::new_gd();
            shape.set_size(size);
            let mut collider = CollisionShape3D::new_alloc();
            collider.set_shape(&shape);

            body.add_child(&collider);
            body.add_child(&mesh_instance);
            self.base_mut().add_child(&body);
        } else {
            mesh_instance.set_position(center);
            self.base_mut().add_child(&mesh_instance);
        }
    }

    fn build_terminal(&mut self, pos: Vector3, label_text: &str, color: Color) -> Gd<Area3D> {
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(Color::from_rgba(0.15, 0.15, 0.17, 1.0));

        let mut screen_mat = StandardMaterial3D::new_gd();
        screen_mat.set_albedo(color);
        screen_mat.set_feature(Feature::EMISSION, true);
        screen_mat.set_emission(color);

        let mut area = Area3D::new_alloc();
        self.base_mut().add_child(&area);
        area.set_global_position(pos);

        let mut podium_mesh = BoxMesh::new_gd();
        podium_mesh.set_size(Vector3::new(0.6, 1.0, 0.4));
        let mut podium = MeshInstance3D::new_alloc();
        podium.set_mesh(&podium_mesh);
        podium.set_surface_override_material(0, &mat);
        podium.set_position(Vector3::new(0.0, 0.5, 0.0));
        area.add_child(&podium);

        let mut screen_mesh = BoxMesh::new_gd();
        screen_mesh.set_size(Vector3::new(0.5, 0.35, 0.05));
        let mut screen = MeshInstance3D::new_alloc();
        screen.set_mesh(&screen_mesh);
        screen.set_surface_override_material(0, &screen_mat);
        screen.set_position(Vector3::new(0.0, 1.15, 0.18));
        screen.set_rotation(Vector3::new(-0.3, 0.0, 0.0));
        area.add_child(&screen);

        let mut label = Label3D::new_alloc();
        label.set_text(label_text);
        label.set_font_size(48);
        label.set_billboard_mode(BillboardMode::ENABLED);
        label.set_position(Vector3::new(0.0, 2.2, 0.0));
        area.add_child(&label);

        let mut shape = BoxShape3D::new_gd();
        shape.set_size(Vector3::new(1.4, 2.0, 1.4));
        let mut collider = CollisionShape3D::new_alloc();
        collider.set_shape(&shape);
        collider.set_position(Vector3::new(0.0, 1.0, 0.0));
        area.add_child(&collider);

        area
    }

    fn build_hud(&mut self) {
        let mut hud = CanvasLayer::new_alloc();

        let mut instructions = Label::new_alloc();
        instructions.set_position(Vector2::new(20.0, 20.0));
        instructions.set_text("Walk into SOLO to begin.");
        hud.add_child(&instructions);

        let mut message_label = Label::new_alloc();
        message_label.set_position(Vector2::new(340.0, 500.0));
        message_label.set_size(Vector2::new(500.0, 60.0));
        message_label.set_text("");
        hud.add_child(&message_label);

        self.base_mut().add_child(&hud);
        self.message_label = Some(message_label);
    }
}
