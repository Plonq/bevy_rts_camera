[![Crates.io](https://img.shields.io/crates/v/bevy_rts_camera)](https://crates.io/crates/bevy_rts_camera)
[![docs.rs](https://docs.rs/bevy_rts_camera/badge.svg)](https://docs.rs/bevy_rts_camera)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

<div style="text-align: center">
  <h1>Bevy RTS Camera</h1>
</div>

[//]: # (![A screen recording showing camera movement]&#40;https://user-images.githubusercontent.com/7709415/230715348-eb19d9a8-4826-4a73-a039-02cacdcb3dc9.gif "Demo of bevy_rts_camera"&#41;)

## Summary

[//]: # (TODO)
Bevy RTS Camera provides RTS-style camera controls for Bevy Engine, to get your game up and running quickly.

## Features:

[//]: # (TODO)

- [x] Smooth panning across ground and follow terrain height
- [x] Zoom
- [x] Edge pan
- [x] rotate around Y (up)
- [x] Lock onto entity (smoothed)
- [x] Custom pan controls
- [x] Custom speed
- [x] Custom edge pan width
- [x] Custom max/min height
- [x] Custom smoothness
- [x] Custom angle
- [x] Snap to location

## Controls

[//]: # (TODO)

## Quick Start

[//]: # (TODO)

Add the plugin:

```rust ignore
.add_plugins(PanOrbitCameraPlugin)
```

Add `PanOrbitCamera` to a camera:

```rust ignore
commands.spawn((
    Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        ..default()
    },
    PanOrbitCamera::default(),
));
```

This will set up a camera with good defaults.

Check out the [advanced example](https://github.com/Plonq/bevy_panorbit_camera/tree/master/examples/advanced.rs) to see
all the possible configuration options.

## Version Compatibility

| bevy | bevy_panorbit_camera |
|------|----------------------|
| 0.14 | 0.1                  |

## License

All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there
are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
