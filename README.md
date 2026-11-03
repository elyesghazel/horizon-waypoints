# Waypoints (Horizon native mod)

Save spots and find your way back to them, in the spirit of Xaero's waypoints:

- **N** saves a waypoint where you stand and opens the menu with its name ready: type one, Enter keeps it.
- **U** opens the **waypoint menu**: every waypoint with its colour, name, coordinates and distance. Click the colour
  square for the next colour, the name or **Rename** to rename it, **x** twice to delete it. At the top: **+ Add
  here**, **Shown** and **Markers**. The wheel scrolls a long list; Esc or U closes it.
- **Markers in the world:** a coloured diamond over each waypoint with its name and distance, drawn where it is on
  screen, through walls and from any distance.
- **A list in a corner:** the nearest waypoints of your dimension, each with an arrow that turns to point at it.
- **Deaths:** dying moves the red `Death` waypoint to where it happened.
- A third key, unbound by default, hides or shows everything. All three keys change in Options > Mods > Waypoints.

Everything also works from chat:

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
`native_tick` (closes the menu when a click asks to), the screen hooks for its menu (`native_screen_render_post`,
`native_screen_mouse_button_pressed_pre`, `native_screen_key_pressed_pre`, `native_screen_char_typed_pre`,
`native_screen_mouse_scrolled_pre`, on the screen `native_mod_screen_open` gives it),
`native_compute_fov` (the camera and field of view the markers are projected with) and `native_health_hud` (it only
reads the health, to see a death, and leaves Horizon's hearts as they are). `compatibility.json` names the hook
version it was built on (80).

| File | What it is |
|---|---|
| `mixins/waypoints/horizon-mixin.toml` | The bundle: one module and the HEAD hooks |
| `mixins/waypoints/runtime.rs` | The logic (storage, commands, the menu's layout, clicks and typing, bearing, projection), added to the game as `game::mod_waypoints`; also this crate's library, for its tests |
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
