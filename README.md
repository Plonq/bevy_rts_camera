[![Crates.io](https://img.shields.io/crates/v/bevy_rts_camera)](https://crates.io/crates/bevy_rts_camera)
[![docs.rs](https://docs.rs/bevy_rts_camera/badge.svg)](https://docs.rs/bevy_rts_camera)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

<div style="text-align: center">
  <h1>Bevy RTS Camera</h1>
</div>

![A screen recording showing camera movement](assets/demo.gif "Demo of bevy_rts_camera")

## Summary

Bevy RTS Camera provides an RTS-style camera for Bevy Engine, to get your game up and running quickly. Designed for
simple use cases, and does not try to cover advanced requirements.

## Features:

- Pan, zoom, and rotation
- Automatically follows whatever you mark as 'ground'
- Smoothed movement
- Customisable controls and other settings

## Controls

Default controls:

- Arrow Keys: pan
- Mouse Wheel: zoom
- Middle Mouse: rotate

You can also 'edge pan' by moving the mouse to the edge of the screen.

## Quick Start

Add the plugin:

```rust ignore
.add_plugins(RtsCameraPlugin)
```

Add `RtsCamera` to a camera:

```rust ignore
commands.spawn((
    Camera3dBundle::default(),
    RtsCamera::default(),
));
```

Add `Ground` to your ground/terrain entities:

```rust ignore
commands.spawn((
    PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(80.0, 80.0)),
        ..default()
    },
    Ground,
));
```

This will set up a camera at world origin with good defaults based on a roughly realistic scale (where an average human
is 1.75 units tall).

Check out the [advanced example](https://github.com/Plonq/bevy_panorbit_camera/tree/master/examples/advanced.rs) to see
all the possible configuration options.

## Version Compatibility

| bevy | bevy_panorbit_camera |
|------|----------------------|
| 0.13 | 0.1                  |

## License

All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there
are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
