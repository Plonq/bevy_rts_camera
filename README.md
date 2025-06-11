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
- Comes with optional controller, or you can control it yourself
- Integrated with [leafwing-input-manager](https://github.com/Leafwing-Studios/leafwing-input-manager/tree/main)

## Default Controller

You can 'edge pan' by moving the mouse to the edge of the screen.



### Minimal Controller 

A minimal controller is included with these default controls:

- Pan: Arrow Keys
- Zoom: Mouse Wheel
- Rotate: Right Mouse + Drag

`RtsCameraAction::minimal_input_map()`
```rust ignore
InputMap::default()
    // Pan Action
    .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::arrow_keys())
    // Zoom Action
    .with_axis(RtsCameraAction::ZoomAxis, MouseScrollAxis::Y)
    // Rotate
    .with(RtsCameraAction::RotateMode, MouseButton::Right)
    .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
```


### Full Controller
`RtsCameraAction::full_input_map()`

A full controller is included with these default controls:
- Pan: Arrow Keys, WASD, Game Pad Button
- Zoom: Mouse Wheel, Key (E,Q)
- Rotate: Right Mouse + Drag, Key (R,F)
- Grab: Middle Mouse + Drag

```rust ignore
InputMap::default()
    // Pan Action
    .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::wasd())
    .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::arrow_keys())
    .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::action_pad())
    // Zoom Action
    .with_axis(
        RtsCameraAction::ZoomAxis,
        VirtualAxis::new(KeyCode::KeyE, KeyCode::KeyQ),
    )
    .with_axis(RtsCameraAction::ZoomAxis, MouseScrollAxis::Y)
    // Rotate
    .with(RtsCameraAction::RotateMode, MouseButton::Right)
    .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
    .with(RtsCameraAction::Rotate(true), KeyCode::KeyR)
    .with(RtsCameraAction::Rotate(false), KeyCode::KeyF)
    // Grab
    .with(RtsCameraAction::GrabMode, MouseButton::Middle)
    .with_dual_axis(RtsCameraAction::GrabAxis, MouseMove::default())
```

## Quick Start

Add the plugin:

```rust ignore
.add_plugins(RtsCameraPlugin)
```

Add `RtsCamera` (this will automatically add a `Camera3d` but you can add it manually if necessary):

```rust ignore
commands.spawn((
    RtsCamera::default(),
    RtsCameraControls::default(), // Optional
    RtsCameraAction::minimal_input_map(), // Minimal Control, Use along with RtsCamera Control
    // RtsCameraAction::full_input_map(), // Full Control
    // You can put a custom leafwing_input_manager::InputMap
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

Check out the [advanced example](https://github.com/Plonq/bevy_rts_camera/blob/main/examples/advanced.rs) to see
the possible configuration options.

## Version Compatibility

| bevy | bevy_rts_camera |
|------|-----------------|
| 0.16 | 0.10            |
| 0.15 | 0.9             |
| 0.14 | 0.8             |
| 0.13 | 0.1-0.7         |

## License

All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there
are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
