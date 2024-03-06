#![warn(missing_docs)]
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
        app.init_resource::<CameraControls>().add_systems(
            Update,
            (
                (initialize, zoom, pan, rotate),
                follow_ground,
                snap_to_target,
                move_towards_target,
                update_camera_transform,
            )
                .chain()
                .in_set(RtsCameraSystemSet),
        );
    }
}

/// System set containing all the systems that control the RTS camera.
/// If you want to control the camera manually in any way (e.g. snapping to a specific location),
/// you should run that before this system set.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RtsCameraSystemSet;

/// RTS camera controls
#[derive(Resource, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CameraControls {
    /// The key that will pan the camera up (or forward).
    /// Defaults to `KeyCode::ArrowUp`.
    pub key_up: KeyCode,
    /// The key that will pan the camera down (or backward).
    /// Defaults to `KeyCode::ArrowDown`.
    pub key_down: KeyCode,
    /// The key that will pan the camera left.
    /// Defaults to `KeyCode::ArrowLeft`.
    pub key_left: KeyCode,
    /// The key that will pan the camera right.
    /// Defaults to `KeyCode::ArrowRight`.
    pub key_right: KeyCode,
    /// The mouse button used to rotate the camera.
    /// Defaults to `MouseButton::Middle`.
    pub button_rotate: MouseButton,
    /// Whether controls are enabled. When disabled, all input will be ignored. However,
    /// you can still control the camera manually, and movement will still be smoothed.
    /// Defaults to `true`.
    pub enabled: bool,
}

impl Default for CameraControls {
    fn default() -> Self {
        CameraControls {
            key_up: KeyCode::ArrowUp,
            key_down: KeyCode::ArrowDown,
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
            button_rotate: MouseButton::Middle,
            enabled: true,
        }
    }
}

/// Marks a camera to be used as an RTS camera.
/// Only one instance of this component should exist at any given moment.
/// Typically you'll add this alongside a `Camera3dBundle`.
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
///             RtsCamera::default(),
///         ));
///  }
/// ```
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct RtsCamera {
    /// How far away from the side of the screen edge pan will kick in, defined as a percentage
    /// of the window's height. Set to `0.0` to disable edge panning.
    /// Defaults to `0.05` (5%).
    pub edge_pan_width: f32,
    /// Speed of camera pan (either via keyboard controls or edge panning).
    /// Defaults to `1.0`.
    pub pan_speed: f32,
    /// The minimum height the camera can zoom in to, or the height of the camera at `1.0` zoom.
    /// Should be set to a value that avoids clipping.
    /// Defaults to `0.5`.
    pub height_min: f32,
    /// The maximum height the camera can zoom out to, or the height of the camera at `0.0` zoom.
    /// Defaults to `10.0`.
    pub height_max: f32,
    /// The angle in radians of the camera, where a value of `0.0` is looking directly down (-Y),
    /// and a value of `TAU / 4.0` (90 degrees) is looking directly forward.
    /// Defaults to 25 degrees.
    pub angle: f32,
    /// The amount of smoothing applied to the camera movement. Should be a value between `0.0` and
    /// `1.0`. Set to `0.0` to disable smoothing. `1.0` is infinite smoothing (the camera won't
    /// move).
    /// Defaults to `0.3`.
    pub smoothness: f32,

    pub transform: Transform,
    pub target_transform: Transform,
    pub zoom: f32,
    pub target_zoom: f32,
    pub snap_next_frame: bool,
}

impl Default for RtsCamera {
    fn default() -> Self {
        RtsCamera {
            edge_pan_width: 0.05,
            pan_speed: 15.0,
            height_min: 2.0,
            height_max: 30.0,
            angle: 20.0f32.to_radians(),
            smoothness: 0.3,
            transform: Transform::IDENTITY,
            target_transform: Transform::IDENTITY,
            zoom: 0.0,
            target_zoom: 0.0,
            snap_next_frame: true,
        }
    }
}

/// Marks an entity that should be treated as 'ground'. The RTS camera will stay a certain distance
/// (based on min/max height and zoom) above any meshes marked with this component (using a ray
/// cast).
/// You'll likely want to mark all terrain entities, but not things like buildings, trees, or units.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Ground;

fn initialize(mut cam_q: Query<&mut RtsCamera, Added<RtsCamera>>) {
    for mut cam in cam_q.iter_mut() {
        // Snap to targets when RtsCamera is added. Note that we snap whole transform, not just XZ
        // translation like snap_to system.
        cam.zoom = cam.target_zoom;
        cam.target_transform.translation.y = cam.height_max.lerp(cam.height_min, cam.zoom);
        cam.transform = cam.target_transform;
    }
}

fn zoom(
    controls: Res<CameraControls>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cam_q: Query<&mut RtsCamera>,
) {
    if !controls.enabled {
        return;
    }

    for mut cam in cam_q.iter_mut() {
        let zoom_amount = mouse_wheel
            .read()
            .map(|event| match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y * 0.001,
            })
            .fold(0.0, |acc, val| acc + val);
        let new_zoom = (cam.target_zoom + zoom_amount * 0.5).clamp(0.0, 1.0);
        cam.target_zoom = new_zoom;
    }
}

#[allow(clippy::too_many_arguments)]
fn pan(
    controls: Res<CameraControls>,
    mut cam_q: Query<&mut RtsCamera>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    if !controls.enabled {
        return;
    }

    for mut cam in cam_q.iter_mut() {
        let mut delta = Vec3::ZERO;

        // Keyboard pan
        if button_input.pressed(controls.key_up) {
            delta += Vec3::from(cam.target_transform.forward())
        }
        if button_input.pressed(controls.key_down) {
            delta += Vec3::from(cam.target_transform.back())
        }
        if button_input.pressed(controls.key_left) {
            delta += Vec3::from(cam.target_transform.left())
        }
        if button_input.pressed(controls.key_right) {
            delta += Vec3::from(cam.target_transform.right())
        }

        // Edge pan
        if delta.length_squared() == 0.0 && !mouse_input.pressed(controls.button_rotate) {
            if let Ok(primary_window) = primary_window_q.get_single() {
                if let Some(cursor_position) = primary_window.cursor_position() {
                    let win_w = primary_window.width();
                    let win_h = primary_window.height();
                    let pan_width = win_h * cam.edge_pan_width;
                    // Pan left
                    if cursor_position.x < pan_width {
                        delta += Vec3::from(cam.target_transform.left())
                    }
                    // Pan right
                    if cursor_position.x > win_w - pan_width {
                        delta += Vec3::from(cam.target_transform.right())
                    }
                    // Pan up
                    if cursor_position.y < pan_width {
                        delta += Vec3::from(cam.target_transform.forward())
                    }
                    // Pan down
                    if cursor_position.y > win_h - pan_width {
                        delta += Vec3::from(cam.target_transform.back())
                    }
                }
            }
        }

        let new_target = cam.target_transform.translation
            + delta.normalize_or_zero()
            * time.delta_seconds()
            * cam.pan_speed
            // Scale based on zoom so it (roughly) feels the same speed at different zoom levels
            * cam.target_zoom.remap(0.0, 1.0, 1.0, 0.5);
        cam.target_transform.translation = new_target;
    }
}

fn rotate(
    mut cam_q: Query<&mut RtsCamera>,
    controls: Res<CameraControls>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !controls.enabled {
        return;
    }

    for mut cam in cam_q.iter_mut() {
        if mouse_input.pressed(controls.button_rotate) {
            let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
            if let Ok(primary_window) = primary_window_q.get_single() {
                // Adjust based on window size, so that moving mouse entire width of window
                // will be one half rotation (180 degrees)
                let delta_x = mouse_delta.x / primary_window.width() * PI;
                cam.target_transform.rotate_local_y(-delta_x);
            }
        }
    }
}

fn follow_ground(
    mut cam_q: Query<&mut RtsCamera>,
    ground_q: Query<Entity, With<Ground>>,
    mut raycast: Raycast,
) {
    for mut cam in cam_q.iter_mut() {
        let ray_start = Vec3::new(
            cam.target_transform.translation.x,
            cam.target_transform.translation.y + cam.height_max,
            cam.target_transform.translation.z,
        );
        if let Some(hit1) = cast_ray(&mut raycast, ray_start, Direction3d::NEG_Y, &|entity| {
            ground_q.get(entity).is_ok()
        }) {
            cam.target_transform.translation.y =
                hit1.position().y + cam.height_max.lerp(cam.height_min, cam.target_zoom);
        }
    }
}

fn snap_to_target(mut cam_q: Query<&mut RtsCamera>) {
    // When snapping in a top down camera, only the XZ should be snapped. The Y coord is controlled
    // by zoom and that should remain smoothed, as should rotation.
    for mut cam in cam_q.iter_mut() {
        if cam.snap_next_frame {
            cam.transform.translation.x = cam.target_transform.translation.x;
            cam.transform.translation.z = cam.target_transform.translation.z;
            cam.snap_next_frame = false;
        }
    }
}

fn move_towards_target(mut cam_q: Query<&mut RtsCamera>, time: Res<Time>) {
    for mut cam in cam_q.iter_mut() {
        cam.transform.translation = cam.transform.translation.lerp(
            cam.target_transform.translation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
        cam.transform.rotation = cam.transform.rotation.lerp(
            cam.target_transform.rotation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
        cam.zoom = cam.zoom.lerp(
            cam.target_zoom,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
    }
}

fn update_camera_transform(mut cam_q: Query<(&mut Transform, &RtsCamera)>) {
    for (mut tfm, cam) in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_x(cam.angle - 90f32.to_radians());
        let camera_offset = (cam.height_max.lerp(cam.height_min, cam.zoom)) * cam.angle.tan();
        tfm.rotation = cam.transform.rotation * rotation;
        tfm.translation = cam.transform.translation + cam.transform.back() * camera_offset;
    }
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
