// todo: re-enable
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::prelude::{Raycast, RaycastSettings};
use std::f32::consts::PI;

/// Bevy plugin that contains the systems for controlling `RtsCamera` components.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, CameraPivot};
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(RtsCameraPlugin)
///         .run();
/// }
/// ```
pub struct RtsCameraPlugin;

impl Plugin for RtsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .init_resource::<CameraConfig>()
            .init_resource::<CameraControls>()
            .add_systems(
                Update,
                (
                    // (
                    //     // initialize,
                    //     zoom,
                    //     follow_ground,
                    //     update_eye_transform,
                    //     pan.run_if(|q: Query<&CameraLock>| q.is_empty()),
                    //     lock.run_if(|q: Query<&CameraLock>| !q.is_empty()),
                    //     rotate,
                    // ),
                    move_towards_target,
                )
                    .chain()
                    .in_set(RtsCameraSystemSet),
            );
    }
}

/// Base system set to allow ordering of `RtsCamera`
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RtsCameraSystemSet;

#[derive(Resource, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CameraControls {
    /// The key that will pan the camera up (or forward)
    pub key_up: KeyCode,
    /// The key that will pan the camera down (or backward)
    pub key_down: KeyCode,
    /// The key that will pan the camera left
    pub key_left: KeyCode,
    /// The key that will pan the camera right
    pub key_right: KeyCode,
    /// The mouse button used to rotate the camera.
    /// Defaults to `MouseButton::Middle`.
    pub button_rotate: MouseButton,
}

impl Default for CameraControls {
    fn default() -> Self {
        CameraControls {
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            button_rotate: MouseButton::Middle,
        }
    }
}

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct CameraConfig {
    /// How far away from the side of the screen edge pan will kick in, defined as a percentage
    /// of the window's height. Set to `0.0` to disable edge panning.
    /// Defaults to `0.05` (5%).
    pub edge_pan_width: f32,
    /// Speed of camera pan (either via keyboard controls or edge panning), measured in units per
    /// second.
    /// Defaults to `1.0`.
    pub speed: f32,
    /// The minimum height the camera can zoom in to. Should be set to a value that avoids clipping.
    /// Defaults to `0.1`.
    pub height_min: f32,
    /// The maximum height the camera can zoom out to.
    /// Defaults to `5.0`.
    pub height_max: f32,
    /// The angle of the camera, where a value of `0.0` is looking directly down (-Y), and a value
    /// of `TAU / 4.0` (90 degrees) is looking directly forward. Measured in radians.
    /// Defaults to 25 degrees.
    pub angle: f32,
    /// The amount of smoothing applied to the camera movement. Should be a value between `0.0` and
    /// `1.0`. Set to `0.0` to disable smoothing. `1.0` is infinite smoothing (the camera won't
    /// move).
    /// Defaults to `0.9`.
    pub smoothness: f32,
    /// Whether `RtsCamera` is enabled. When disabled, all input will be ignored, but it will still
    /// move towards the `target`.
    pub enabled: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig {
            edge_pan_width: 0.05,
            speed: 1.0,
            height_min: 0.1,
            height_max: 5.0,
            angle: 25.0f32.to_radians(),
            smoothness: 0.3,
            enabled: true,
        }
    }
}

#[derive(Resource, Debug, PartialEq, Clone)]
struct CameraState {
    /// The current zoom level of the camera, defined as a percentage of the distance between the
    /// minimum height and maximum height. A value of `1.0` is 100% zoom (min height), and a value
    /// of `0.0` is 0% zoom (maximum height). Automatically updated when zooming with the scroll
    /// wheel.
    /// Defaults to `0.0`.
    zoom: f32,
    /// The target position of the camera (the entity with `RtsCamera` specifically). The camera's
    /// actual position will move towards this based on `smoothness` every frame.
    /// Automatically updated based upon inputs.
    /// Defaults to `Vec3::ZERO`.
    target: Vec3,
    /// Whether the camera has initialized. This is primarily used when the camera is first added
    /// to the scene, so it can snap to its starting position, ignoring any smoothing.
    /// Defaults to `false`.
    initialized: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            target: Vec3::ZERO,
            zoom: 0.0,
            initialized: false,
        }
    }
}

/// Tags an entity as capable of panning and orbiting, and provides a way to configure the
/// camera's behaviour and controls.
/// The entity must have `Transform` and `Projection` components. Typically you would add a
/// `Camera3dBundle` which already contains these.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, CameraPivot};
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(DefaultPlugins)
/// #         .add_plugins(RtsCameraPlugin)
/// #         .add_systems(Startup, setup)
/// #         .run();
/// # }
/// fn setup(mut commands: Commands) {
///     commands
///         .spawn((
///             Camera3dBundle::default(),
///             CameraPivot::default(),
///         ));
///  }
/// ```
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CameraPivot;

// impl RtsCamera {
//     /// Return the current height of the camera based on the min/max height and the zoom level
//     fn height(&self) -> f32 {
//         self.height_max.lerp(self.height_min, self.zoom)
//     }
//
//     /// Return the distance offset to the camera based on the angle and the current height.
//     /// I.e. this is how far the camera is from what the camera is looking at, ignoring the Y
//     /// axis.
//     fn camera_offset(&self) -> f32 {
//         self.height() * self.angle.tan()
//     }
// }

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CameraEye;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CameraLock;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Ground;

// fn initialize(
//     mut rts_camera: Query<(&Transform, &mut RtsCameraPivot, &Children)>,
//     mut rts_cam_eye: Query<&mut Transform, (With<RtsCameraEye>, Without<RtsCameraPivot>)>,
// ) {
//     for (rts_cam_tfm, mut rts_cam, children) in
//         rts_camera.iter_mut().filter(|(_, cam, _)| !cam.initialized)
//     {
//         rts_cam.target = rts_cam_tfm.translation;
//         for child in children {
//             if let Ok(mut eye_tfm) = rts_cam_eye.get_mut(*child) {
//                 eye_tfm.rotation = Quat::from_rotation_x(rts_cam.angle - 90f32.to_radians());
//                 eye_tfm.translation.z = rts_cam.camera_offset();
//             }
//         }
//         rts_cam.initialized = true;
//     }
// }

fn zoom(
    config: Res<CameraConfig>,
    mut state: ResMut<CameraState>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    if !config.enabled {
        return;
    }

    let zoom_amount = mouse_wheel
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.001,
        })
        .fold(0.0, |acc, val| acc + val);
    let new_zoom = (state.zoom + zoom_amount * 0.5).clamp(0.0, 1.0);
    state.zoom = new_zoom;
}

fn update_eye_transform(
    config: Res<CameraConfig>,
    state: Res<CameraState>,
    rts_camera: Query<&Children, Without<CameraEye>>,
    mut rts_cam_eye: Query<&mut Transform, With<CameraEye>>,
    time: Res<Time>,
) {
    if !config.enabled {
        return;
    }

    for children in rts_camera.iter() {
        for child in children {
            if let Ok(mut eye_tfm) = rts_cam_eye.get_mut(*child) {
                eye_tfm.rotation = Quat::from_rotation_x(config.angle - 90f32.to_radians());
                let camera_offset =
                    (config.height_max.lerp(config.height_min, state.zoom)) * config.angle.tan();
                eye_tfm.translation.z = eye_tfm.translation.z.lerp(
                    camera_offset,
                    1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn pan(
    config: Res<CameraConfig>,
    controls: Res<CameraControls>,
    mut state: ResMut<CameraState>,
    rts_camera: Query<&Transform>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    if !config.enabled {
        return;
    }

    for rts_cam_tfm in rts_camera.iter() {
        let mut delta = Vec3::ZERO;

        // Keyboard pan
        if button_input.pressed(controls.key_up) {
            delta += Vec3::from(rts_cam_tfm.forward())
        }
        if button_input.pressed(controls.key_down) {
            delta += Vec3::from(rts_cam_tfm.back())
        }
        if button_input.pressed(controls.key_left) {
            delta += Vec3::from(rts_cam_tfm.left())
        }
        if button_input.pressed(controls.key_right) {
            delta += Vec3::from(rts_cam_tfm.right())
        }

        // Edge pan
        if delta.length_squared() == 0.0 && !mouse_input.pressed(controls.button_rotate) {
            if let Ok(primary_window) = primary_window_q.get_single() {
                if let Some(cursor_position) = primary_window.cursor_position() {
                    let win_w = primary_window.width();
                    let win_h = primary_window.height();
                    let pan_width = win_h * config.edge_pan_width;
                    // Pan left
                    if cursor_position.x < pan_width {
                        delta += Vec3::from(rts_cam_tfm.left())
                    }
                    // Pan right
                    if cursor_position.x > win_w - pan_width {
                        delta += Vec3::from(rts_cam_tfm.right())
                    }
                    // Pan up
                    if cursor_position.y < pan_width {
                        delta += Vec3::from(rts_cam_tfm.forward())
                    }
                    // Pan down
                    if cursor_position.y > win_h - pan_width {
                        delta += Vec3::from(rts_cam_tfm.back())
                    }
                }
            }
        }

        let new_target =
            state.target + delta.normalize_or_zero() * time.delta_seconds() * 2.0 * config.speed;
        state.target = new_target;
    }
}

fn lock(
    config: Res<CameraConfig>,
    mut state: ResMut<CameraState>,
    target: Query<(&Transform, &CameraLock), Without<CameraPivot>>,
) {
    if !config.enabled {
        return;
    }

    for (target_tfm, _lock) in target.iter() {
        state.target.x = target_tfm.translation.x;
        state.target.z = target_tfm.translation.z;
    }
}

fn follow_ground(
    config: Res<CameraConfig>,
    mut state: ResMut<CameraState>,
    rts_camera: Query<&Transform>,
    ground_q: Query<Entity, With<Ground>>,
    mut gizmos: Gizmos,
    mut raycast: Raycast,
) {
    if !config.enabled {
        return;
    }

    for rts_cam_tfm in rts_camera.iter() {
        // todo: add more rays to smooth transition between sudden ground height changes???
        // Ray starting directly above where the camera is looking, pointing straight down
        let ray1 = Ray3d::new(rts_cam_tfm.translation, Vec3::from(rts_cam_tfm.down()));
        let hits1 = raycast.debug_cast_ray(
            ray1,
            &RaycastSettings {
                filter: &|entity| ground_q.get(entity).is_ok(),
                ..default()
            },
            &mut gizmos,
        );
        let hit1 = hits1.first().map(|(_, hit)| hit);
        if let Some(hit1) = hit1 {
            state.target.y =
                hit1.position().y + config.height_max.lerp(config.height_min, state.zoom);
        }
    }
}

fn rotate(
    config: Res<CameraConfig>,
    controls: Res<CameraControls>,
    mut rts_camera: Query<&mut Transform>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !config.enabled {
        return;
    }

    for mut rts_cam_tfm in rts_camera.iter_mut() {
        if mouse_input.pressed(controls.button_rotate) {
            let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
            if let Ok(primary_window) = primary_window_q.get_single() {
                // Adjust based on window size, so that moving mouse entire width of window
                // will be one half rotation (180 degrees)
                let delta_x = mouse_delta.x / primary_window.width() * PI;
                rts_cam_tfm.rotate_local_y(-delta_x);
            }
        }
    }
}

fn move_towards_target(
    state: Res<CameraState>,
    config: Res<CameraConfig>,
    mut rts_camera: Query<&mut Transform>,
    time: Res<Time>,
) {
    for mut rts_cam_tfm in rts_camera.iter_mut() {
        rts_cam_tfm.translation = rts_cam_tfm.translation.lerp(
            state.target,
            1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
        );
    }
}
