# Coordinates HUD

Horizon's official Coordinates HUD: your block position and the direction you face in a box in the top-left corner of
the screen. It looks as 2.0.0 did: `XYZ -123.4 / 76.0 / 0.0` and `Facing North (-Z)`, white shadowed text on
translucent black, 2 GUI pixels from the edge, scaled with the GUI.

- `/where` puts your coordinates and dimension in chat (`-124, 76, -1 in minecraft:overworld`).
- `/coords` hides or shows the box, and the game keeps your choice.

A coordinate just below zero reads `0.0` in the box and `-1` in `/where`, never `-0.0` or `-0`. The facing turns half
way between two sides, as vanilla's `Direction.fromYRot`.

## Install

In the Horizon launcher: **Mods**, find Coordinates HUD, **Install**. It is a WebAssembly mod, so nothing is built on
your computer.

## Settings

**Options > Mods > Coordinates HUD > Config:** Shown (on). `/coords` switches the same setting. Kept in
`horizon-mods/.horizon-mod-data/coords_hud/shown` (`true`/`false`), the same file earlier versions used, so a box you
hid in 0.1.0 or 2.0.0 stays hidden.

The game gives no Options > Horizon Extras settings for Coordinates HUD (`official_extra_settings` has none for it), so
nothing is carried over from `horizon-extras.txt`.

While the debug screen (F3) is open the box gives way to it, as 2.0.0 did: F3 shows the coordinates itself. The time of
day line 0.1.0 had stays gone, as in 2.0.0.

## Commands

`/where` and `/coords` are client commands: the chat box completes them, and they run on your computer, so the server
never sees them. Both work on any server, and in single player without cheats.

## Build

With [`cargo horizon-mod`](https://github.com/horizon-rust/horizon/tree/main/docs/modding):

```sh
cargo test                       # the lines, the facing, /where's line and the settings file
cargo horizon-mod build --wasm   # target/horizon-mod/coords_hud.hmod
cargo horizon-mod check
```

## Earlier versions (legacy)

2.0.0 was a native `.hnmod` built into the game by the launcher, and 0.1.0 an API 0.1 WebAssembly mod. Launchers from
0.2.70 on can't load either, and their install and build steps no longer apply. Their source is in this folder's
history.

## Licence

MIT, see [LICENSE](LICENSE).
