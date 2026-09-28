use godot::classes::node::ProcessMode;
use godot::classes::{CanvasLayer, ColorRect, INode3D, Input, Label, Node3D, ProgressBar};
use godot::global::Key;
use godot::prelude::*;

use crate::abomination::Abomination;
use crate::extraction_door::ExtractionDoor;
use crate::level;
use crate::maze::Maze;
use crate::pickup::{AmmoPickup, Coin};
use crate::player::Player;
use crate::wall_creature::{Resolution, WallCreature};

const MAZE_SIZE: i32 = 7;
const FIXATION_THRESHOLD: f64 = 40.0;
const ABOMINATION_THRESHOLD: f64 = 60.0;
const WALLCREATURE_QUICK_DREAD: f64 = 8.0;
const WALLCREATURE_TIMEOUT_DREAD: f64 = 20.0;
const COIN_COUNT: usize = 10;
const AMMO_COUNT: usize = 5;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct GameRoot {
    base: Base<Node3D>,

    player: Option<Gd<Player>>,
    wall_creature: Option<Gd<WallCreature>>,
    abomination: Option<Gd<Abomination>>,
    wall_points: Vec<Vector3>,

    fixation: f64,
    dread: f64,
    game_over: bool,
    prev_r_held: bool,

    paused: bool,
    prev_escape_held: bool,
    prev_q_held: bool,

    coins_label: Option<Gd<Label>>,
    ammo_label: Option<Gd<Label>>,
    fixation_bar: Option<Gd<ProgressBar>>,
    dread_bar: Option<Gd<ProgressBar>>,
    message_label: Option<Gd<Label>>,
    pause_label: Option<Gd<Label>>,
    hallucination_overlay: Option<Gd<ColorRect>>,
}

#[godot_api]
impl INode3D for GameRoot {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player: None,
            wall_creature: None,
            abomination: None,
            wall_points: Vec::new(),
            fixation: 0.0,
            dread: 0.0,
            game_over: false,
            prev_r_held: false,
            paused: false,
            prev_escape_held: false,
            prev_q_held: false,
            coins_label: None,
            ammo_label: None,
            fixation_bar: None,
            dread_bar: None,
            message_label: None,
            pause_label: None,
            hallucination_overlay: None,
        }
    }

    fn ready(&mut self) {
        // Keep processing while the tree is paused, so this node can still see
        // the Escape key that un-pauses it — gameplay children (Player,
        // enemies) stay on the default PAUSABLE mode and freeze normally.
        self.base_mut().set_process_mode(ProcessMode::ALWAYS);

        let maze = Maze::generate(MAZE_SIZE, MAZE_SIZE);

        let mut level_parent = Node3D::new_alloc();
        self.base_mut().add_child(&level_parent);
        let info = level::build(&mut level_parent, &maze);
        self.wall_points = info.wall_points;

        let mut player = Player::new_alloc();
        self.base_mut().add_child(&player);
        player.set_global_position(info.spawn + Vector3::new(0.0, 0.1, 0.0));
        self.player = Some(player.clone());

        let mut door = ExtractionDoor::new_alloc();
        self.base_mut().add_child(&door);
        door.set_global_position(info.extraction);
        door.bind_mut().player = Some(player.clone());

        let mut spots = info.item_spots.clone();
        shuffle(&mut spots);

        for _ in 0..COIN_COUNT.min(spots.len()) {
            if let Some(pos) = spots.pop() {
                let mut coin = Coin::new_alloc();
                self.base_mut().add_child(&coin);
                coin.set_global_position(pos + Vector3::new(0.0, 0.9, 0.0));
                coin.bind_mut().player = Some(player.clone());
            }
        }
        for _ in 0..AMMO_COUNT.min(spots.len()) {
            if let Some(pos) = spots.pop() {
                let mut ammo = AmmoPickup::new_alloc();
                self.base_mut().add_child(&ammo);
                ammo.set_global_position(pos + Vector3::new(0.0, 0.7, 0.0));
                ammo.bind_mut().player = Some(player.clone());
            }
        }

        self.build_hud();
    }

    fn process(&mut self, delta: f64) {
        self.handle_pause_input();
        if self.paused {
            return;
        }

        if self.game_over {
            self.handle_restart();
            return;
        }

        let Some(player) = self.player.clone() else { return };

        let pending = player.bind().pending_fixation;
        if pending != 0.0 {
            self.fixation = (self.fixation + pending).clamp(0.0, 100.0);
            player.clone().bind_mut().pending_fixation = 0.0;
        }

        if self.wall_creature.is_none()
            && self.abomination.is_none()
            && self.fixation >= FIXATION_THRESHOLD
        {
            self.spawn_wall_creature(&player);
            self.fixation = 0.0;
        }

        if let Some(wc) = self.wall_creature.clone() {
            let resolution = wc.bind().resolution;
            match resolution {
                Resolution::Active => {}
                Resolution::Killed => {
                    self.dread = (self.dread + WALLCREATURE_QUICK_DREAD).min(100.0);
                    self.wall_creature = None;
                    wc.free();
                }
                Resolution::TimedOut => {
                    self.dread = (self.dread + WALLCREATURE_TIMEOUT_DREAD).min(100.0);
                    self.wall_creature = None;
                    wc.free();
                }
            }
        }

        if self.abomination.is_none() && self.dread >= ABOMINATION_THRESHOLD {
            self.spawn_abomination(&player);
        }

        self.update_hud(delta);

        if player.bind().is_dead {
            self.show_end_screen("YOU WERE TAKEN\n\nPress R to try again");
            self.game_over = true;
            self.release_mouse();
        } else if player.bind().reached_extraction {
            let coins = player.bind().coins;
            self.show_end_screen(&format!(
                "EXTRACTED\n\nCoins collected: {coins}\n\nPress R to go again"
            ));
            self.game_over = true;
            self.release_mouse();
        }
    }
}

impl GameRoot {
    fn spawn_wall_creature(&mut self, player: &Gd<Player>) {
        let player_pos = player.bind().global_pos();
        let Some(nearest) = self
            .wall_points
            .iter()
            .copied()
            .min_by(|a, b| (*a - player_pos).length().total_cmp(&(*b - player_pos).length()))
        else {
            return;
        };

        let mut creature = WallCreature::new_alloc();
        self.base_mut().add_child(&creature);
        creature.set_global_position(nearest);
        creature.bind_mut().player = Some(player.clone());
        self.wall_creature = Some(creature);
    }

    fn spawn_abomination(&mut self, player: &Gd<Player>) {
        let Some(&spawn_point) = self.wall_points.first() else {
            return;
        };
        let mut abomination = Abomination::new_alloc();
        self.base_mut().add_child(&abomination);
        abomination.set_global_position(spawn_point);
        abomination.bind_mut().player = Some(player.clone());
        self.abomination = Some(abomination);
    }

    fn build_hud(&mut self) {
        let mut hud = CanvasLayer::new_alloc();

        let mut coins_label = Label::new_alloc();
        coins_label.set_position(Vector2::new(20.0, 20.0));
        coins_label.set_text("Coins: 0");
        hud.add_child(&coins_label);

        let mut ammo_label = Label::new_alloc();
        ammo_label.set_position(Vector2::new(20.0, 46.0));
        ammo_label.set_text("Ammo: 0");
        hud.add_child(&ammo_label);

        let mut fixation_label = Label::new_alloc();
        fixation_label.set_position(Vector2::new(20.0, 76.0));
        fixation_label.set_text("Fixation");
        hud.add_child(&fixation_label);

        let mut fixation_bar = ProgressBar::new_alloc();
        fixation_bar.set_position(Vector2::new(20.0, 100.0));
        fixation_bar.set_size(Vector2::new(220.0, 18.0));
        fixation_bar.set_min(0.0);
        fixation_bar.set_max(100.0);
        hud.add_child(&fixation_bar);

        let mut dread_label = Label::new_alloc();
        dread_label.set_position(Vector2::new(20.0, 128.0));
        dread_label.set_text("Dread");
        hud.add_child(&dread_label);

        let mut dread_bar = ProgressBar::new_alloc();
        dread_bar.set_position(Vector2::new(20.0, 152.0));
        dread_bar.set_size(Vector2::new(220.0, 18.0));
        dread_bar.set_min(0.0);
        dread_bar.set_max(100.0);
        hud.add_child(&dread_bar);

        let mut message_label = Label::new_alloc();
        message_label.set_position(Vector2::new(340.0, 240.0));
        message_label.set_size(Vector2::new(500.0, 240.0));
        message_label.set_text("");
        hud.add_child(&message_label);

        let mut pause_label = Label::new_alloc();
        pause_label.set_position(Vector2::new(340.0, 240.0));
        pause_label.set_size(Vector2::new(500.0, 120.0));
        pause_label.set_text("");
        hud.add_child(&pause_label);

        let mut overlay = ColorRect::new_alloc();
        overlay.set_position(Vector2::new(0.0, 0.0));
        overlay.set_size(Vector2::new(2000.0, 2000.0));
        overlay.set_color(Color::from_rgba(0.5, 0.0, 0.0, 0.0));
        hud.add_child(&overlay);

        self.base_mut().add_child(&hud);

        self.coins_label = Some(coins_label);
        self.ammo_label = Some(ammo_label);
        self.fixation_bar = Some(fixation_bar);
        self.dread_bar = Some(dread_bar);
        self.message_label = Some(message_label);
        self.pause_label = Some(pause_label);
        self.hallucination_overlay = Some(overlay);
    }

    fn update_hud(&mut self, _delta: f64) {
        let Some(player) = self.player.clone() else { return };
        let p = player.bind();

        if let Some(l) = self.coins_label.as_mut() {
            l.set_text(&format!("Coins: {}", p.coins));
        }
        if let Some(l) = self.ammo_label.as_mut() {
            l.set_text(&format!("Ammo: {}", p.ammo));
        }
        if let Some(b) = self.fixation_bar.as_mut() {
            b.set_value(self.fixation);
        }
        if let Some(b) = self.dread_bar.as_mut() {
            b.set_value(self.dread);
        }
        if let Some(o) = self.hallucination_overlay.as_mut() {
            let alpha = if p.is_hallucinating() { 0.35 } else { 0.0 };
            o.set_color(Color::from_rgba(0.5, 0.0, 0.0, alpha));
        }
    }

    fn show_end_screen(&mut self, text: &str) {
        if let Some(l) = self.message_label.as_mut() {
            l.set_text(text);
        }
    }

    fn release_mouse(&mut self) {
        Input::singleton().set_mouse_mode(godot::classes::input::MouseMode::VISIBLE);
    }

    fn handle_pause_input(&mut self) {
        if self.game_over {
            return;
        }

        let input = Input::singleton();
        let escape_held = input.is_key_pressed(Key::ESCAPE);
        if escape_held && !self.prev_escape_held {
            self.paused = !self.paused;
            self.base().get_tree().set_pause(self.paused);

            if self.paused {
                Input::singleton().set_mouse_mode(godot::classes::input::MouseMode::VISIBLE);
                if let Some(l) = self.pause_label.as_mut() {
                    l.set_text("PAUSED\n\nEsc: Resume\nQ: Quit to menu");
                }
            } else {
                Input::singleton().set_mouse_mode(godot::classes::input::MouseMode::CAPTURED);
                if let Some(l) = self.pause_label.as_mut() {
                    l.set_text("");
                }
            }
        }
        self.prev_escape_held = escape_held;

        if self.paused {
            let q_held = input.is_key_pressed(Key::Q);
            if q_held && !self.prev_q_held {
                let mut tree = self.base().get_tree();
                tree.set_pause(false);
                let _ = tree.change_scene_to_file("res://main.tscn");
            }
            self.prev_q_held = q_held;
        }
    }

    fn handle_restart(&mut self) {
        let held = Input::singleton().is_key_pressed(Key::R);
        if held && !self.prev_r_held {
            let _ = self.base().get_tree().reload_current_scene();
        }
        self.prev_r_held = held;
    }
}

fn shuffle(items: &mut Vec<Vector3>) {
    for i in (1..items.len()).rev() {
        let j = rand::random_range(0..=i);
        items.swap(i, j);
    }
}
