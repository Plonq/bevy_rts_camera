#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::f32::consts::TAU;

use bevy::math::bounding::Aabb2d;
use bevy::picking::mesh_picking::ray_cast::RayMeshHit;
use bevy::prelude::*;

pub use controller::RtsCameraControls;

use crate::controller::RtsCameraControlsPlugin;

mod controller;

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
                    apply_camera_bounds,
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
///             RtsCamera::default(),
///         ));
///  }
/// ```
#[derive(Component, Copy, Clone, Debug)]
#[require(Camera3d)]
pub struct RtsCamera {
    /// The minimum height the camera can zoom in to, or the height of the camera at `1.0` zoom.
    /// Should be set to a value that avoids clipping.
    /// Defaults to `0.5`.
    pub height_min: f32,
    /// The maximum height the camera can zoom out to, or the height of the camera at `0.0` zoom.
    /// Defaults to `10.0`.
    pub height_max: f32,
    /// The bounds in which the camera is constrained, along the XZ plane of `target_focus`. This
    /// prevents panning past these limits. Imagine looking directly down relative to `target_focus`
    /// and the XZ plane corresponds XY of the Vec2s, except +Y is up/forward (-Z).
    /// Defaults to `Aabb2d::new(Vec2::ZERO, Vec2::new(20.0, 20.0))` (i.e. can move 20.0 in any
    /// direction starting at world center).
    pub bounds: Aabb2d,
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
            bounds: Aabb2d::new(Vec2::ZERO, Vec2::new(20.0, 20.0)),
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

impl RtsCamera {
    /// Sets the camera's position, angle and focus immediately to their current smoothing destination.
    pub fn reset_smoothing(&mut self) {
        self.focus.translation = self.target_focus.translation;
        self.focus.rotation = self.target_focus.rotation;
        self.zoom = self.target_zoom;
        self.angle = self.target_angle;
    }
}

/// Marks an entity that should be treated as 'ground'. The RTS camera will stay a certain distance
/// (based on min/max height and zoom) above any meshes marked with this component (using a ray
/// cast).
/// You'll likely want to mark all terrain entities, but not things like buildings, trees, or units.
#[derive(Component, Copy, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Ground;

fn initialize(mut cam_q: Query<&mut RtsCamera, Added<RtsCamera>>) {
    for mut cam in cam_q.iter_mut() {
        // Snap to targets when RtsCamera is added. Note that we snap whole transform, not just XZ
        // translation like snap_to system.
        cam.zoom = cam.target_zoom;
        cam.focus = cam.target_focus;
        cam.angle = cam.min_angle;
        cam.target_angle = cam.min_angle;
    }
}

fn follow_ground(
    mut cam_q: Query<&mut RtsCamera>,
    ground_q: Query<Entity, With<Ground>>,
    mut ray_cast: MeshRayCast,
) {
    for mut cam in cam_q.iter_mut() {
        let ray_start = Vec3::new(
            cam.target_focus.translation.x,
            cam.target_focus.translation.y + cam.height_max,
            cam.target_focus.translation.z,
        );
        if let Some(hit1) = cast_ray(ray_start, Dir3::NEG_Y, &mut ray_cast, &|entity| {
            ground_q.get(entity).is_ok()
        }) {
            cam.target_focus.translation.y = hit1.point.y;
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
        cam.target_angle = cam
            .min_angle
            .lerp(MAX_ANGLE, ease_in_circular(cam.target_zoom));
    }
}

fn move_towards_target(mut cam_q: Query<&mut RtsCamera>, time: Res<Time<Real>>) {
    for mut cam in cam_q.iter_mut() {
        cam.focus.translation = cam.focus.translation.lerp(
            cam.target_focus.translation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_secs()),
        );
        cam.focus.rotation = cam.focus.rotation.lerp(
            cam.target_focus.rotation,
            1.0 - cam.smoothness.powi(7).powf(time.delta_secs()),
        );
        cam.zoom = cam.zoom.lerp(
            cam.target_zoom,
            1.0 - cam.smoothness.powi(7).powf(time.delta_secs()),
        );
        cam.angle = cam.angle.lerp(
            cam.target_angle,
            1.0 - cam.smoothness.powi(7).powf(time.delta_secs()),
        );
    }
}

/// Constrains a 3D position to lie within the specified 2D axis-aligned
/// bounding box (AABB).
///
/// This function ensures that the X and Z coordinates of the given 3D position
/// are clamped to the bounds of the provided [`Aabb2d`]. The Y coordinate
/// remains unchanged, as the bounds only apply to the XZ plane.
#[inline(always)]
fn apply_bounds(bounds: &Aabb2d, position: Vec3) -> Vec3 {
    let closest_point = bounds.closest_point(Vec2::new(position.x, position.z));

    Vec3::new(closest_point.x, position.y, closest_point.y)
}

fn apply_camera_bounds(mut cam_q: Query<&mut RtsCamera>) {
    for mut cam in cam_q.iter_mut() {
        cam.target_focus.translation = apply_bounds(&cam.bounds, cam.target_focus.translation);
    }
}

fn update_camera_transform(mut cam_q: Query<(&mut Transform, &RtsCamera)>) {
    for (mut tfm, cam) in cam_q.iter_mut() {
        let rotation = Quat::from_rotation_x(cam.angle - 90f32.to_radians());
        let camera_height = cam.height_max.lerp(cam.height_min, cam.zoom);
        let camera_offset = camera_height * cam.angle.tan();

        tfm.rotation = cam.focus.rotation * rotation;
        tfm.translation =
            cam.focus.translation + (Vec3::Y * camera_height) + (cam.focus.back() * camera_offset);
    }
}

fn cast_ray<'a>(
    origin: Vec3,
    dir: Dir3,
    ray_cast: &'a mut MeshRayCast<'_, '_>,
    filter: &'a dyn Fn(Entity) -> bool,
) -> Option<&'a RayMeshHit> {
    let ray1 = Ray3d::new(origin, dir);
    let hits1 = ray_cast.cast_ray(
        ray1,
        &MeshRayCastSettings {
            filter,
            ..default()
        },
    );
    hits1.first().map(|(_, hit)| hit)
}

fn ease_in_circular(x: f32) -> f32 {
    1.0 - (1.0 - x.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use bevy::math::bounding::Aabb2d;

    use super::*;

    #[test]
    fn test_symmetric_bounds() {
        // Default bounds with zero center, i.e., symmetric around origin.
        let bounds = Aabb2d::new(Vec2::ZERO, Vec2::new(20.0, 20.0));

        // Position at center should remain unchanged.
        let center_pos = Vec3::new(0.0, 0.0, 0.0);
        assert_eq!(apply_bounds(&bounds, center_pos), center_pos);

        // Position at edge should remain unchanged.
        let edge_pos = Vec3::new(20.0, 0.0, 20.0);
        assert_eq!(apply_bounds(&bounds, edge_pos), edge_pos);

        // Position inside bounds should remain unchanged.
        let in_bounds_pos = Vec3::new(15.0, 0.0, 15.0);
        assert_eq!(apply_bounds(&bounds, in_bounds_pos), in_bounds_pos);

        // Position outside bounds should be clamped.
        let out_of_bounds_pos = Vec3::new(25.0, 0.0, 25.0);
        assert_eq!(
            apply_bounds(&bounds, out_of_bounds_pos),
            Vec3::new(20.0, 0.0, 20.0)
        );
    }

    #[test]
    fn test_asymmetric_bounds() {
        // Bounds with non-zero center, i.e., asymmetric relative to origin.
        //
        // Effective bounds: x: [10, 190], z: [-10, 190].
        let bounds = Aabb2d::new(Vec2::new(100.0, 90.0), Vec2::new(90.0, 100.0));

        // Position at center should remain unchanged.
        let center_pos = Vec3::new(100.0, 0.0, 90.0);
        assert_eq!(apply_bounds(&bounds, center_pos), center_pos);

        // Position at edge should remain unchanged.
        let edge_pos = Vec3::new(190.0, 0.0, 190.0);
        assert_eq!(apply_bounds(&bounds, edge_pos), edge_pos);

        // Position inside bounds should remain unchanged.
        let in_bounds_pos = Vec3::new(150.0, 0.0, 100.0);
        assert_eq!(apply_bounds(&bounds, in_bounds_pos), in_bounds_pos);

        // Position outside bounds should be clamped
        let out_of_bounds_pos = Vec3::new(200.0, 0.0, 200.0); // beyond edge
        assert_eq!(
            apply_bounds(&bounds, out_of_bounds_pos),
            Vec3::new(190.0, 0.0, 190.0)
        );
    }
}
