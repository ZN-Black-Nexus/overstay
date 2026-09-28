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

## Getting a test build

Push a tag matching `v*` (e.g. `v0.1.0`) and [.github/workflows/release.yml](.github/workflows/release.yml)
builds a debug arm64 Linux `.deb` and a debug amd64 Windows build (zipped —
the `.exe` needs its sibling `.dll` alongside it, so it can't ship as a bare
`.exe`), then publishes both as a GitHub **pre-release** on this repo:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Grab the files from the release page once the workflow finishes (Actions tab
to watch progress). This is the standing way to get a build onto a device to
test — every future test build should go out this way rather than a local
export.
