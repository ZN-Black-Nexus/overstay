# Overstay

Co-op / solo 3D horror game. See [GAME_DESIGN.md](GAME_DESIGN.md) for the
full design.

## Layout

- `godot/` — Godot 4.7 project (scenes, assets, the `.gdextension` that
  loads the Rust library).
- `rust/` — the gdextension crate. All gameplay logic lives here, written
  in Rust against [gdext](https://github.com/godot-rust/gdext).

## Building

```bash
cd rust
cargo build
```

Then open `godot/project.godot` in the Godot 4.7 editor (or run
`godot --path godot`) — it loads the compiled library automatically via
`overstay.gdextension`.
