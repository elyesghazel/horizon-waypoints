# Waypoints (Horizon native mod)

Save spots and find your way back to them, in the spirit of Xaero's waypoints:

- **Markers in the world:** a coloured diamond over each waypoint with its name and distance, drawn where it is on
  screen, through walls and from any distance.
- **A list in a corner:** the nearest waypoints of your dimension, each with an arrow that turns to point at it.
- **A key:** **N** (Options > Mods > Waypoints to change it) saves a waypoint where you stand. A second key, unbound by
  default, hides or shows them all.
- **Deaths:** dying moves the red `Death` waypoint to where it happened.

| Command | What it does |
|---|---|
| `/wp add [name] [colour]` | A waypoint where you stand (`/wp add Base gold`); the same name again moves it |
| `/wp remove <name or number>` | Removes one (`/wp remove 2`) |
| `/wp list` | Every waypoint with its number, coordinates and distance |
| `/wp toggle` | Hides or shows them |
| `/wp clear confirm` | Removes every waypoint |

Colours: `red green blue yellow gold aqua pink white`, or `#rrggbb`. Without one, each new waypoint gets the next.

Settings (Options > Mods > Waypoints > Settings): shown, markers in the world, the list's length (3, 5, 8 or off),
its corner, and the death waypoint.

Waypoints are kept per dimension in `horizon-mods/.horizon-mod-data/waypoints/list`, one per line (name, dimension,
x, y, z, colour), shared by every world and server you play: name them apart if you play several.

## Install

In the Horizon launcher: **Mods > Add from GitHub**, paste `https://github.com/elyesghazel/horizon-waypoints`, then
**Build** (you read the source and trust it: native mods run unsandboxed, so the launcher asks). From a terminal:
`horizon-launcher mods add-github https://github.com/elyesghazel/horizon-waypoints --trust-native-code`. The first
build downloads the launcher's own Rust and takes a while. See Horizon's
[native mods](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md).

## Made on the hook points

It hooks only Horizon's native hook points (`crates/horizon-client/src/game/native_hooks.rs`): `native_mods`,
`native_settings`, `native_settings_click`, `native_hud`, `native_command`, `native_commands`, `native_key`,
`native_compute_fov` (the camera and field of view the markers are projected with) and `native_health_hud` (it only
reads the health, to see a death, and leaves Horizon's hearts as they are). `compatibility.json` names the hook
version it was built on (80).

| File | What it is |
|---|---|
| `mixins/waypoints/horizon-mixin.toml` | The bundle: one module and the HEAD hooks |
| `mixins/waypoints/runtime.rs` | The logic (storage, commands, bearing, projection), added to the game as `game::mod_waypoints`; also this crate's library, for its tests |
| `mixins/waypoints/*.rs` | One small helper per hook |

## Build

```sh
cargo test        # the logic: storage, commands, bearing, projection
```

To check it against the game, it also lives in a fork of the mod registry, where Horizon's own tooling applies it on
the pinned Horizon commit and runs `cargo check` on the whole client:
[elyesghazel/horizon-mods, branch `waypoints`](https://github.com/elyesghazel/horizon-mods/tree/waypoints/native/waypoints)
(`python3 native/tools/native.py check waypoints`, then `package waypoints` for the `.hnmod`).

License: MIT.
