// todo: re-enable
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::f32::consts::PI;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::prelude::{IntersectionData, Raycast, RaycastSettings};

/// Bevy plugin that provides RTS camera controls.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin};
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
        app.init_resource::<CameraConfig>()
            .init_resource::<CameraControls>()
            .init_resource::<CameraSnapTo>()
            .init_resource::<CameraTargetTransform>()
            .init_resource::<CameraActualTransform>()
            .init_resource::<CameraTargetZoom>()
            .init_resource::<CameraActualZoom>()
            .add_systems(
                Update,
                (
                    zoom,
                    pan.run_if(resource_equals(CameraSnapTo(false))),
                    rotate,
                    follow_ground,
                    snap_to_target.run_if(resource_equals(CameraSnapTo(true))),
                    move_towards_target,
                    update_camera_transform,
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
    pub pan_speed: f32,
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
            pan_speed: 1.0,
            height_min: 0.1,
            height_max: 5.0,
            angle: 25.0f32.to_radians(),
            smoothness: 0.3,
            enabled: true,
        }
    }
}

#[derive(Resource, Debug, PartialEq, Clone)]
pub struct CameraSnapTo(pub bool);

impl Default for CameraSnapTo {
    fn default() -> Self {
        CameraSnapTo(true)
    }
}

#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub struct CameraTargetTransform(pub Transform);

#[derive(Resource, Default, Debug, PartialEq, Clone)]
struct CameraActualTransform(Transform);

#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub struct CameraTargetZoom(pub f32);

#[derive(Resource, Default, Debug, PartialEq, Clone)]
struct CameraActualZoom(f32);

/// Tags an entity as capable of panning and orbiting, and provides a way to configure the
/// camera's behaviour and controls.
/// The entity must have `Transform` and `Projection` components. Typically you would add a
/// `Camera3dBundle` which already contains these.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, RtsCamera};
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
///             RtsCamera,
///         ));
///  }
/// ```
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct RtsCamera;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Ground;

fn zoom(
    config: Res<CameraConfig>,
    mut target_zoom: ResMut<CameraTargetZoom>,
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
    let new_zoom = (target_zoom.0 + zoom_amount * 0.5).clamp(0.0, 1.0);
    target_zoom.0 = new_zoom;
}

#[allow(clippy::too_many_arguments)]
fn pan(
    config: Res<CameraConfig>,
    controls: Res<CameraControls>,
    mut target_tfm: ResMut<CameraTargetTransform>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    if !config.enabled {
        return;
    }

    let mut delta = Vec3::ZERO;

    // Keyboard pan
    if button_input.pressed(controls.key_up) {
        delta += Vec3::from(target_tfm.0.forward())
    }
    if button_input.pressed(controls.key_down) {
        delta += Vec3::from(target_tfm.0.back())
    }
    if button_input.pressed(controls.key_left) {
        delta += Vec3::from(target_tfm.0.left())
    }
    if button_input.pressed(controls.key_right) {
        delta += Vec3::from(target_tfm.0.right())
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
                    delta += Vec3::from(target_tfm.0.left())
                }
                // Pan right
                if cursor_position.x > win_w - pan_width {
                    delta += Vec3::from(target_tfm.0.right())
                }
                // Pan up
                if cursor_position.y < pan_width {
                    delta += Vec3::from(target_tfm.0.forward())
                }
                // Pan down
                if cursor_position.y > win_h - pan_width {
                    delta += Vec3::from(target_tfm.0.back())
                }
            }
        }
    }

    let new_target = target_tfm.0.translation
        + delta.normalize_or_zero() * time.delta_seconds() * 2.0 * config.pan_speed;
    target_tfm.0.translation = new_target;
}

fn follow_ground(
    config: Res<CameraConfig>,
    mut target_tfm: ResMut<CameraTargetTransform>,
    target_zoom: ResMut<CameraTargetZoom>,
    ground_q: Query<Entity, With<Ground>>,
    mut raycast: Raycast,
) {
    if !config.enabled {
        return;
    }

    let ray_start = Vec3::new(
        target_tfm.0.translation.x,
        target_tfm.0.translation.y + config.height_max,
        target_tfm.0.translation.z,
    );
    if let Some(hit1) = cast_ray(&mut raycast, ray_start, Direction3d::NEG_Y, &|entity| {
        ground_q.get(entity).is_ok()
    }) {
        target_tfm.0.translation.y =
            hit1.position().y + config.height_max.lerp(config.height_min, target_zoom.0);
    }
}

fn rotate(
    mut target_tfm: ResMut<CameraTargetTransform>,
    config: Res<CameraConfig>,
    controls: Res<CameraControls>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !config.enabled {
        return;
    }

    if mouse_input.pressed(controls.button_rotate) {
        let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
        if let Ok(primary_window) = primary_window_q.get_single() {
            // Adjust based on window size, so that moving mouse entire width of window
            // will be one half rotation (180 degrees)
            let delta_x = mouse_delta.x / primary_window.width() * PI;
            target_tfm.0.rotate_local_y(-delta_x);
        }
    }
}

fn update_camera_transform(
    zoom: Res<CameraActualZoom>,
    tfm: Res<CameraActualTransform>,
    config: Res<CameraConfig>,
    mut camera_q: Query<&mut Transform, With<RtsCamera>>,
) {
    if let Ok(mut camera) = camera_q.get_single_mut() {
        let rotation = Quat::from_rotation_x(config.angle - 90f32.to_radians());
        let camera_offset =
            (config.height_max.lerp(config.height_min, zoom.0)) * config.angle.tan();
        camera.rotation = tfm.0.rotation * rotation;
        camera.translation = tfm.0.translation + tfm.0.back() * camera_offset;
    }
}

fn move_towards_target(
    mut tfm: ResMut<CameraActualTransform>,
    target_tfm: Res<CameraTargetTransform>,
    mut zoom: ResMut<CameraActualZoom>,
    target_zoom: Res<CameraTargetZoom>,
    config: Res<CameraConfig>,
    time: Res<Time>,
) {
    tfm.0.translation = tfm.0.translation.lerp(
        target_tfm.0.translation,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
    tfm.0.rotation = tfm.0.rotation.lerp(
        target_tfm.0.rotation,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
    zoom.0 = zoom.0.lerp(
        target_zoom.0,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
}

fn snap_to_target(
    mut tfm: ResMut<CameraActualTransform>,
    target_tfm: Res<CameraTargetTransform>,
    mut snap_to: ResMut<CameraSnapTo>,
) {
    // When snapping in a top down camera, only the XZ should be snapped. The Y coord is controlled
    // by zoom and that should remain smoothed, as should rotation.
    tfm.0.translation.x = target_tfm.0.translation.x;
    tfm.0.translation.z = target_tfm.0.translation.z;
    snap_to.0 = false;
}

fn cast_ray<'a>(
    raycast: &'a mut Raycast<'_, '_>,
    origin: Vec3,
    dir: Direction3d,
    filter: &'a dyn Fn(Entity) -> bool,
) -> Option<&'a IntersectionData> {
    let ray1 = Ray3d::new(origin, Vec3::from(dir));
    let hits1 = raycast.cast_ray(
        ray1,
        &RaycastSettings {
            filter,
            ..default()
        },
    );
    hits1.first().map(|(_, hit)| hit)
}
