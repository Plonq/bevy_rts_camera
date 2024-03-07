#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;
use bevy_mod_raycast::prelude::{IntersectionData, Raycast, RaycastSettings};

mod controller;
use crate::controller::RtsCameraControllerPlugin;
pub use controller::RtsCameraController;

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
        app.add_plugins(RtsCameraControllerPlugin)
            .add_systems(PreUpdate, initialize)
            .add_systems(
                Update,
                (
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

/// Marks a camera to be used as an RTS camera.
/// Only one instance of this component should exist at any given moment.
/// Typically you'll add this alongside a `Camera3dBundle`.
/// This does not include a controller. Add `RtsCameraController` as well if you want.
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
    /// The angle in radians of the camera, where a value of `0.0` is looking directly down (-Y),
    /// and a value of `TAU / 4.0` (90 degrees) is looking directly forward.
    /// Defaults to 25 degrees.
    pub angle: f32,
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
    /// Updated automatically when using `RtsCameraController`, but should be updated manually
    /// if you implement your own controls. You can also change this when adding this component to
    /// set the starting position.
    pub target_focus: Transform,
    /// The current zoom level, between `0.0` and `1.0`, where 0 is no zoom (`height_max`), and 1 is
    /// max zoom (`height_min`).
    /// Typically you won't need to set this manually, even if you implement your own controls.
    /// Set `target_zoom` instead.
    /// Defaults to `0.0`.
    pub zoom: f32,
    /// The target zoom level. Used to implement zoom smoothing.
    /// Updated automatically when using `RtsCameraController`, but should be updated manually
    /// if you implement your own controls. You can also change this when adding this component to
    /// set the starting zoom.
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
    }
}

fn update_camera_transform(mut cam_q: Query<(&mut Transform, &RtsCamera)>) {
    for (mut tfm, cam) in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_x(cam.angle - 90f32.to_radians());
        let camera_offset = (cam.height_max.lerp(cam.height_min, cam.zoom)) * cam.angle.tan();
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
