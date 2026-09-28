use godot::classes::base_material_3d::TextureParam;
use godot::classes::light_3d::Param as LightParam;
use godot::classes::{
    BoxMesh, BoxShape3D, CollisionShape3D, Environment, MeshInstance3D, Node3D, OmniLight3D,
    StandardMaterial3D, StaticBody3D, WorldEnvironment,
};
use godot::classes::environment::AmbientSource;
use godot::prelude::*;

use crate::maze::Maze;
use crate::texgen::generate_texture;

pub const CELL_SIZE: f32 = 4.0;
pub const WALL_HEIGHT: f32 = 3.0;
const WALL_THICKNESS: f32 = 0.2;

pub struct LevelInfo {
    pub spawn: Vector3,
    pub extraction: Vector3,
    /// World-space points along walls, candidates for an environmental-possession eruption.
    pub wall_points: Vec<Vector3>,
    /// World-space cell-center points where coins/ammo can be placed (spawn cell excluded).
    pub item_spots: Vec<Vector3>,
}

fn cell_center(x: i32, y: i32) -> Vector3 {
    Vector3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE)
}

fn add_block(
    parent: &mut Gd<Node3D>,
    center: Vector3,
    size: Vector3,
    material: &Gd<StandardMaterial3D>,
    solid: bool,
) {
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
        parent.add_child(&body);
    } else {
        mesh_instance.set_position(center);
        parent.add_child(&mesh_instance);
    }
}

/// Builds the 3D geometry for a generated maze under `parent`, and reports spawn info
/// the caller (GameRoot) needs: where the player starts, where extraction is, and
/// candidate points for item/creature placement.
pub fn build(parent: &mut Gd<Node3D>, maze: &Maze) -> LevelInfo {
    let wall_mat = {
        // Muted khaki-yellow "damp wallpaper" — blob water stains plus faint
        // vertical seams where wallpaper rolls would meet.
        let tex = generate_texture(
            128,
            Color::from_rgba(0.72, 0.64, 0.36, 1.0),
            Color::from_rgba(0.33, 0.27, 0.13, 1.0),
            6,
            None,
            Some((Color::from_rgba(0.5, 0.44, 0.22, 1.0), 32)),
        );
        let mut m = StandardMaterial3D::new_gd();
        m.set_albedo(Color::from_rgba(0.72, 0.64, 0.36, 1.0));
        if let Some(t) = tex {
            m.set_texture(TextureParam::ALBEDO, &t);
        }
        m
    };

    let floor_mat = {
        let tex = generate_texture(
            128,
            Color::from_rgba(0.30, 0.24, 0.13, 1.0),
            Color::from_rgba(0.16, 0.13, 0.06, 1.0),
            12,
            None,
            None,
        );
        let mut m = StandardMaterial3D::new_gd();
        m.set_albedo(Color::from_rgba(0.30, 0.24, 0.13, 1.0));
        if let Some(t) = tex {
            m.set_texture(TextureParam::ALBEDO, &t);
        }
        m
    };

    let ceiling_mat = {
        let tex = generate_texture(
            128,
            Color::from_rgba(0.78, 0.76, 0.62, 1.0),
            Color::from_rgba(0.55, 0.53, 0.42, 1.0),
            4,
            Some((Color::from_rgba(0.5, 0.5, 0.4, 1.0), 32)),
            None,
        );
        let mut m = StandardMaterial3D::new_gd();
        m.set_albedo(Color::from_rgba(0.78, 0.76, 0.62, 1.0));
        if let Some(t) = tex {
            m.set_texture(TextureParam::ALBEDO, &t);
        }
        m
    };

    // Faint ambient fill so the maze isn't pitch black; sparse warm lights add the
    // flickery fluorescent-office feel on top.
    let mut env = Environment::new_gd();
    env.set_ambient_source(AmbientSource::COLOR);
    env.set_ambient_light_color(Color::from_rgba(0.55, 0.5, 0.35, 1.0));
    env.set_ambient_light_energy(0.55);
    let mut world_env = WorldEnvironment::new_alloc();
    world_env.set_environment(&env);
    parent.add_child(&world_env);

    let mut wall_points = Vec::new();
    let mut item_spots = Vec::new();

    for y in 0..maze.h {
        for x in 0..maze.w {
            let center = cell_center(x, y);
            let cell = maze.cell(x, y);

            add_block(
                parent,
                center + Vector3::new(0.0, -0.05, 0.0),
                Vector3::new(CELL_SIZE, 0.1, CELL_SIZE),
                &floor_mat,
                true,
            );
            add_block(
                parent,
                center + Vector3::new(0.0, WALL_HEIGHT, 0.0),
                Vector3::new(CELL_SIZE, 0.1, CELL_SIZE),
                &ceiling_mat,
                false,
            );

            if (x + y) % 3 == 0 {
                let mut light = OmniLight3D::new_alloc();
                light.set_position(center + Vector3::new(0.0, WALL_HEIGHT - 0.3, 0.0));
                light.set_color(Color::from_rgba(0.95, 0.9, 0.7, 1.0));
                light.set_param(LightParam::ENERGY, 2.2);
                light.set_param(LightParam::RANGE, CELL_SIZE * 2.2);
                parent.add_child(&light);
            } else {
                item_spots.push(center);
            }

            // North wall: owned by this cell.
            if !cell.open[0] {
                let p = center + Vector3::new(0.0, WALL_HEIGHT * 0.5, -CELL_SIZE * 0.5);
                add_block(
                    parent,
                    p,
                    Vector3::new(CELL_SIZE, WALL_HEIGHT, WALL_THICKNESS),
                    &wall_mat,
                    true,
                );
                wall_points.push(p);
            }
            // West wall: owned by this cell.
            if !cell.open[3] {
                let p = center + Vector3::new(-CELL_SIZE * 0.5, WALL_HEIGHT * 0.5, 0.0);
                add_block(
                    parent,
                    p,
                    Vector3::new(WALL_THICKNESS, WALL_HEIGHT, CELL_SIZE),
                    &wall_mat,
                    true,
                );
                wall_points.push(p);
            }
            // South boundary wall (only on the last row).
            if y == maze.h - 1 && !cell.open[2] {
                let p = center + Vector3::new(0.0, WALL_HEIGHT * 0.5, CELL_SIZE * 0.5);
                add_block(
                    parent,
                    p,
                    Vector3::new(CELL_SIZE, WALL_HEIGHT, WALL_THICKNESS),
                    &wall_mat,
                    true,
                );
                wall_points.push(p);
            }
            // East boundary wall (only on the last column).
            if x == maze.w - 1 && !cell.open[1] {
                let p = center + Vector3::new(CELL_SIZE * 0.5, WALL_HEIGHT * 0.5, 0.0);
                add_block(
                    parent,
                    p,
                    Vector3::new(WALL_THICKNESS, WALL_HEIGHT, CELL_SIZE),
                    &wall_mat,
                    true,
                );
                wall_points.push(p);
            }
        }
    }

    let start = (0, 0);
    let extraction_cell = maze.farthest_from(start);

    item_spots.retain(|p| *p != cell_center(start.0, start.1));

    LevelInfo {
        spawn: cell_center(start.0, start.1),
        extraction: cell_center(extraction_cell.0, extraction_cell.1),
        wall_points,
        item_spots,
    }
}
