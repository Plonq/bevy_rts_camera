#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;
use bevy_mod_raycast::prelude::{IntersectionData, Raycast, RaycastSettings};
use std::f32::consts::TAU;

mod controller;
use crate::controller::RtsCameraControlsPlugin;
pub use controller::RtsCameraControls;

const MAX_ANGLE: f32 = TAU / 5.0;

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
        app.add_plugins(RtsCameraControlsPlugin)
            .add_systems(PreUpdate, initialize)
            .add_systems(
                Update,
                (
                    follow_ground,
                    snap_to_target,
                    dynamic_angle,
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

/// Marks a camera to be used as an RTS camera.
/// Only one instance of this component should exist at any given moment.
/// Typically you'll add this alongside a `Camera3dBundle`.
/// This does not include a controller. Add `RtsCameraControls` as well if you want.
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
    /// The minimum height the camera can zoom in to, or the height of the camera at `1.0` zoom.
    /// Should be set to a value that avoids clipping.
    /// Defaults to `0.5`.
    pub height_min: f32,
    /// The maximum height the camera can zoom out to, or the height of the camera at `0.0` zoom.
    /// Defaults to `10.0`.
    pub height_max: f32,
    /// The current angle in radians of the camera, where a value of `0.0` is looking directly down
    /// (-Y), and a value of `TAU / 4.0` (90 degrees) is looking directly forward.
    /// If you want to customise the angle, set `min_angle` instead.
    /// Defaults to 25 degrees.
    pub angle: f32,
    /// The target angle in radians of the camera, where a value of `0.0` is looking directly down
    /// (-Y), and a value of `TAU / 4.0` (90 degrees) is looking directly forward.
    /// The camera will smoothly transition from `angle` to `target_angle`.
    /// If you want to customise the angle, set `min_angle` instead.
    /// Defaults to 25 degrees.
    pub target_angle: f32,
    /// The angle of the camera at no zoom (max height). By default, angle increases as you zoom in.
    /// If `dynamic_angle` is disabled, then that does not happen and the camera will stay fixed at
    /// `min_zoom`.
    /// If you want to customise the angle, this is what you want to change.
    /// Defaults to 25 degrees.
    pub min_angle: f32,
    /// Whether the camera should increase its angle the more you zoom in, so you can see
    /// characters up close from a sideways view instead of top down.
    /// If this is
    /// Defaults to `true`.
    pub dynamic_angle: bool,
    /// The amount of smoothing applied to the camera movement. Should be a value between `0.0` and
    /// `1.0`. Set to `0.0` to disable smoothing. `1.0` is infinite smoothing (the camera won't
    /// move).
    /// Defaults to `0.3`.
    pub smoothness: f32,
    /// The current focus of the camera, including the orientation (which way is forward). The
    /// camera's actual transform is calculated based on this transform.
    /// Updated automatically.
    /// Typically you won't need to set this manually, even if you implement your own controls.
    /// Set `target_focus` instead.
    /// Defaults to `Transform::IDENTITY`.
    pub focus: Transform,
    /// The target focus of the camera, including the target orientation (which way is forward).
    /// This is where the camera should move to, and is how smoothing is implemented
    /// Updated automatically when using `RtsCameraControls`, but should be updated manually
    /// if you implement your own controls. You can also change this when adding this component to
    /// set the starting position.
    /// Defaults to `Transform::IDENTITY`.
    pub target_focus: Transform,
    /// The current zoom level, between `0.0` and `1.0`, where 0 is no zoom (`height_max`), and 1 is
    /// max zoom (`height_min`).
    /// Typically you won't need to set this manually, even if you implement your own controls.
    /// Set `target_zoom` instead.
    /// Defaults to `0.0`.
    pub zoom: f32,
    /// The target zoom level. Used to implement zoom smoothing.
    /// Updated automatically when using `RtsCameraControls`, but should be updated manually
    /// if you implement your own controls. You can also change this when adding this component to
    /// set the starting zoom.
    /// Defaults to `0.0`.
    pub target_zoom: f32,
    /// Whether the camera should snap to `target_focus` and `target_zoom`. Will be set to
    /// `false` after one frame. Useful if you want to lock the camera to a specific target (e.g.
    /// to follow a unit), by setting `target_focus` and setting this to `true` on every frame.
    /// Defaults to `false`.
    pub snap: bool,
}

impl Default for RtsCamera {
    fn default() -> Self {
        RtsCamera {
            height_min: 2.0,
            height_max: 30.0,
            angle: 20.0f32.to_radians(),
            target_angle: 20.0f32.to_radians(),
            min_angle: 20.0f32.to_radians(),
            dynamic_angle: true,
            smoothness: 0.3,
            focus: Transform::IDENTITY,
            target_focus: Transform::IDENTITY,
            zoom: 0.0,
            target_zoom: 0.0,
            snap: false,
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
        cam.target_focus.translation.y = cam.height_max.lerp(cam.height_min, cam.zoom);
        cam.focus = cam.target_focus;
        cam.angle = cam.min_angle;
        cam.target_angle = cam.min_angle;
    }
}

fn follow_ground(
    mut cam_q: Query<&mut RtsCamera>,
    ground_q: Query<Entity, With<Ground>>,
    mut raycast: Raycast,
) {
    for mut cam in cam_q.iter_mut() {
        let ray_start = Vec3::new(
            cam.target_focus.translation.x,
            cam.target_focus.translation.y + cam.height_max,
            cam.target_focus.translation.z,
        );
        if let Some(hit1) = cast_ray(&mut raycast, ray_start, Direction3d::NEG_Y, &|entity| {
            ground_q.get(entity).is_ok()
        }) {
            cam.target_focus.translation.y =
                hit1.position().y + cam.height_max.lerp(cam.height_min, cam.target_zoom);
        }
    }
}

fn snap_to_target(mut cam_q: Query<&mut RtsCamera>) {
    // When snapping in a top down camera, only the XZ should be snapped. The Y coord is controlled
    // by zoom and that should remain smoothed, as should rotation.
    for mut cam in cam_q.iter_mut() {
        if cam.snap {
            cam.focus.translation.x = cam.target_focus.translation.x;
            cam.focus.translation.z = cam.target_focus.translation.z;
            cam.snap = false;
        }
    }
}

fn dynamic_angle(mut query: Query<&mut RtsCamera>) {
    for mut cam in query.iter_mut().filter(|cam| cam.dynamic_angle) {
        cam.target_angle = cam.min_angle.lerp(MAX_ANGLE, ease_in_circular(cam.zoom));
    }
}

fn move_towards_target(mut cam_q: Query<&mut RtsCamera>, time: Res<Time>) {
    for mut cam in cam_q.iter_mut() {
        cam.focus.translation = cam.focus.translation.lerp(
            cam.target_focus.translation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
        cam.focus.rotation = cam.focus.rotation.lerp(
            cam.target_focus.rotation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
        cam.zoom = cam.zoom.lerp(
            cam.target_zoom,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
        cam.angle = cam.target_angle.lerp(
            cam.target_angle,
            1.0 - cam.smoothness.powi(7).powf(time.delta_seconds()),
        );
    }
}

fn update_camera_transform(mut cam_q: Query<(&mut Transform, &RtsCamera)>) {
    for (mut tfm, cam) in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_x(cam.angle - 90f32.to_radians());
        let mut camera_offset = (cam.height_max.lerp(cam.height_min, cam.zoom)) * cam.angle.tan();
        if cam.dynamic_angle {
            // Subtract up to half of the offset, so the camera gets closer to the target
            // without ending up sitting on top of it (i.e. to get a nice front view)
            camera_offset *= 1.0 - ease_in_circular(cam.zoom).remap(0.0, 1.0, 0.0, 0.4);
        }
        tfm.rotation = cam.focus.rotation * rotation;
        tfm.translation = cam.focus.translation + cam.focus.back() * camera_offset;
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

fn ease_in_circular(x: f32) -> f32 {
    1.0 - (1.0 - x.powi(2)).sqrt()
}
