use godot::prelude::*;

mod abomination;
mod extraction_door;
mod game;
mod level;
mod maze;
mod pickup;
mod player;
mod texgen;
mod wall_creature;

struct OverstayExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OverstayExtension {}
