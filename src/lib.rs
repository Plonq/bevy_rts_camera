// todo: re-enable
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::f32::consts::PI;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::prelude::{IntersectionData, Raycast, RaycastSettings};

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
                    zoom,
                    pan,
                    follow_ground,
                    rotate,
                    update_camera_transform,
                    // snap_to_target.run_if(|state: Res<CameraState>| !state.initialized),
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
pub struct CameraState {
    /// The current zoom level of the camera, defined as a percentage of the distance between the
    /// minimum height and maximum height. A value of `1.0` is 100% zoom (min height), and a value
    /// of `0.0` is 0% zoom (maximum height). Automatically updated when zooming with the scroll
    /// wheel.
    /// Defaults to `0.0`.
    zoom: f32,
    target_zoom: f32,
    /// The target position of the camera (the entity with `RtsCamera` specifically). The camera's
    /// actual position will move towards this based on `smoothness` every frame.
    /// Automatically updated based upon inputs.
    /// Defaults to `Vec3::ZERO`.
    pub target: Transform,
    pub target_target: Transform,
    /// Whether the camera has initialized. This is primarily used when the camera is first added
    /// to the scene, so it can snap to its starting position, ignoring any smoothing.
    /// Defaults to `false`.
    initialized: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            target: Transform::IDENTITY,
            target_target: Transform::IDENTITY,
            zoom: 0.0,
            target_zoom: 0.0,
            initialized: false,
        }
    }
}

impl CameraState {
    pub fn jump_to(&mut self, target: Vec3) {
        // self.target = target;
    }

    pub fn snap_to(&mut self, target: Vec3) {
        self.jump_to(target);
        // Take advantage of the fact that snapping to target is the same thing that happens on
        // initialization
        self.initialized = false;
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
pub struct RtsCamera;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CameraPivot;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CameraEye;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Ground;

fn update_camera_transform(
    state: Res<CameraState>,
    config: Res<CameraConfig>,
    mut camera_q: Query<&mut Transform, With<RtsCamera>>,
) {
    if let Ok(mut camera) = camera_q.get_single_mut() {
        let rotation = Quat::from_rotation_x(config.angle - 90f32.to_radians());
        let camera_offset =
            (config.height_max.lerp(config.height_min, state.zoom)) * config.angle.tan();
        camera.rotation = state.target.rotation * rotation;
        camera.translation = state.target.translation + state.target.back() * camera_offset;
    }
}

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
    let new_zoom = (state.target_zoom + zoom_amount * 0.5).clamp(0.0, 1.0);
    state.target_zoom = new_zoom;
}

#[allow(clippy::too_many_arguments)]
fn pan(
    config: Res<CameraConfig>,
    controls: Res<CameraControls>,
    mut state: ResMut<CameraState>,
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
        delta += Vec3::from(state.target_target.forward())
    }
    if button_input.pressed(controls.key_down) {
        delta += Vec3::from(state.target_target.back())
    }
    if button_input.pressed(controls.key_left) {
        delta += Vec3::from(state.target_target.left())
    }
    if button_input.pressed(controls.key_right) {
        delta += Vec3::from(state.target_target.right())
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
                    delta += Vec3::from(state.target_target.left())
                }
                // Pan right
                if cursor_position.x > win_w - pan_width {
                    delta += Vec3::from(state.target_target.right())
                }
                // Pan up
                if cursor_position.y < pan_width {
                    delta += Vec3::from(state.target_target.forward())
                }
                // Pan down
                if cursor_position.y > win_h - pan_width {
                    delta += Vec3::from(state.target_target.back())
                }
            }
        }
    }

    let new_target = state.target_target.translation
        + delta.normalize_or_zero() * time.delta_seconds() * 2.0 * config.pan_speed;
    state.target_target.translation = new_target;
}

fn follow_ground(
    config: Res<CameraConfig>,
    mut state: ResMut<CameraState>,
    ground_q: Query<Entity, With<Ground>>,
    mut raycast: Raycast,
) {
    if !config.enabled {
        return;
    }

    let ray_start = Vec3::new(
        state.target_target.translation.x,
        state.target_target.translation.y + config.height_max,
        state.target_target.translation.z,
    );
    if let Some(hit1) = cast_ray(&mut raycast, ray_start, Direction3d::NEG_Y, &|entity| {
        ground_q.get(entity).is_ok()
    }) {
        state.target_target.translation.y =
            hit1.position().y + config.height_max.lerp(config.height_min, state.target_zoom);
    }
}

fn rotate(
    mut state: ResMut<CameraState>,
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
            state.target_target.rotate_local_y(-delta_x);
        }
    }
}

fn move_towards_target(mut state: ResMut<CameraState>, config: Res<CameraConfig>, time: Res<Time>) {
    state.target.translation = state.target.translation.lerp(
        state.target_target.translation,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
    state.target.rotation = state.target.rotation.lerp(
        state.target_target.rotation,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
    state.zoom = state.zoom.lerp(
        state.target_zoom,
        1.0 - config.smoothness.powi(7).powf(time.delta_seconds()),
    );
}

fn snap_to_target(
    mut state: ResMut<CameraState>,
    config: Res<CameraConfig>,
    mut pivot_q: Query<&mut Transform, With<CameraPivot>>,
    ground_q: Query<Entity, With<Ground>>,
    mut raycast: Raycast,
) {
    let Ok(mut pivot) = pivot_q.get_single_mut() else {
        return;
    };

    if let Some(hit1) = cast_ray(&mut raycast, pivot.translation, pivot.down(), &|entity| {
        ground_q.get(entity).is_ok()
    }) {
        state.target.translation.y =
            hit1.position().y + config.height_max.lerp(config.height_min, state.zoom);
    }

    pivot.translation = state.target.translation;

    state.initialized = true;
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
