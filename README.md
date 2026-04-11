# Orbital

A simple space flight simulator.

## Keybinds

### Vessel

Key | Action
--- | ---
<kbd>a</kbd> | Rotate counter-clockwise
<kbd>d</kbd> | Rotate clockwise
<kbd>L-Shift</kbd> | Increase throttle
<kbd>L-Ctrl</kbd> | Decrease throttle
<kbd>z</kbd> | Full throttle
<kbd>x</kbd> | Cut throttle
<kbd>[</kbd> | Previous vessel
<kbd>]</kbd> | Next vessel
<kbd>t</kbd> | Toggle SAS
<kbd>p</kbd> | SAS Prograde
<kbd>r</kbd> | SAS Retrograde
<kbd>i</kbd> | SAS Radial-in
<kbd>o</kbd> | SAS Radial-out
<kbd>Space</kbd> | Fire photon

### Camera

Key | Action
--- | ---
<kbd>-</kbd> | Zoom out
<kbd>=</kbd> | Zoom in
<kbd>Up</kbd> | Pan camera up
<kbd>Left</kbd> | Pan camera left
<kbd>Right</kbd> | Pan camera right
<kbd>Down</kbd> | Pan camera down
<kbd>F4</kbd> | Unlock camera
<kbd>v</kbd> | Next camera view

### Time

Key | Action
--- | ---
<kbd>,</kbd> | Time Warp decrease
<kbd>.</kbd> | Time Warp increase
<kbd>/</kbd> | Pause

### HUD

Key | Action
--- | ---
<kbd>F8</kbd> | Show/hide orbits
<kbd>F12</kbd> | Show/hide developer tools

## Development

### Setting up a development environment

A nix flake is provided for easily setting up a reproducible build environment.

TODO: Instructions to enable new nix command and flakes

```bash
$ nix develop
```

### Build/Run

Use `--features dev` to dynamically link Bevy for fast build times.

```bash
$ cargo run --features dev
```

### Formatting

```bash
$ rustfmt \
      --config=imports_granularity=Module,group_imports=StdExternalCrate \
      --edition=2024 \
      src/*
```
