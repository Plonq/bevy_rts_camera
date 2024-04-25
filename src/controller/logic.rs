use std::f32::consts::PI;

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::RtsCamera;

/// An abstraction over the camera zoom. Modify this resource to zoom the camera.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct DeltaZoom {
    /// The entity to act upon. If `None`, will affect all instances of `RtsCamera`
    pub entity: Option<Entity>,
    /// The amount to zoom this frame.
    /// Will be automatically zeroed in `RtsCameraControlsBaseSystemSet` and thus
    /// should be updated before that system set.
    pub delta: f32,
}

/// An abstraction over the camera pan. Modify this resource to pan the camera.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct DeltaPan {
    /// The entity to act upon. If `None`, will affect all instances of `RtsCamera`
    pub entity: Option<Entity>,
    /// The amount to pan this frame, in world space. Only X and Z components are
    /// used. Oriented to the camera, where -Z is forward. E.g. Vec3::new(0.0, 0.0, 1.0)
    /// will pan the camera one unit forward relative to the camera's current orientation.
    /// Will be automatically zeroed in `RtsCameraControlsBaseSystemSet` and thus
    /// should be updated before that system set.
    pub delta: Vec3,
}

/// An abstraction over the camera pan. Different from `DeltaPan` in that `delta`
/// should be the delta of mouse movement when 'dragging', in screen space.
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct DeltaGrab {
    /// The entity to act upon. If `None`, will affect all instances of `RtsCamera`
    pub entity: Option<Entity>,
    /// The amount to pan this frame, in screen space. For example to implement 'grab
    /// and drag' style panning, this value should be updated from `MouseMotion` events
    /// (i.e. in screen space) while the user is 'grabbing'.
    /// Will be automatically zeroed in `RtsCameraControlsBaseSystemSet` and thus
    /// should be updated before that system set.
    pub delta: Vec2,
    /// The point on the 'ground' that the user grabbed. Panning movement will be scaled
    /// in order to (roughly) keep the point on the ground under the cursor.
    /// If this is `None`, the current `target_focus` is used instead, which will result in
    /// the movement feeling higher sensitivity when grabbed closer to the camera and lower
    /// sensitivity when grabbed further from the camera.
    pub grab_pos: Option<Vec3>,
}

pub fn delta_zoom(mut delta_zoom: ResMut<DeltaZoom>, mut cam_q: Query<(Entity, &mut RtsCamera)>) {
    if delta_zoom.delta == 0.0 {
        return;
    }

    for (entity, mut cam) in cam_q.iter_mut() {
        if delta_zoom.entity.is_some_and(|e| e != entity) {
            continue;
        }

        let new_zoom = (cam.target_zoom + delta_zoom.delta * 0.5).clamp(0.0, 1.0);
        cam.target_zoom = new_zoom;

        delta_zoom.delta = 0.0;
    }
}

pub fn delta_pan(mut delta_pan: ResMut<DeltaPan>, mut cam_q: Query<(Entity, &mut RtsCamera)>) {
    if delta_pan.delta == Vec3::ZERO {
        return;
    }

    for (entity, mut cam) in cam_q.iter_mut() {
        if delta_pan.entity.is_some_and(|e| e != entity) {
            continue;
        }

        let focus_delta = delta_pan.delta.x * Vec3::from(cam.target_focus.right())
            + delta_pan.delta.z * Vec3::from(cam.target_focus.back());

        let new_target = cam.target_focus.translation
            + focus_delta
            // Scale based on zoom so it (roughly) feels the same speed at different zoom levels
            * cam.target_zoom.remap(0.0, 1.0, 1.0, 0.5);
        cam.target_focus.translation = new_target;

        delta_pan.delta = Vec3::ZERO;
    }
}

pub fn delta_grab(
    mut delta_grab: ResMut<DeltaGrab>,
    mut cam_q: Query<(Entity, &Transform, &mut RtsCamera, &Camera, &Projection)>,
) {
    if delta_grab.delta == Vec2::ZERO {
        return;
    }

    for (entity, cam_tfm, mut cam, camera, projection) in cam_q.iter_mut() {
        if delta_grab.entity.is_some_and(|e| e != entity) {
            continue;
        }

        let mut delta = delta_grab.delta;

        let mut multiplier = 1.0;
        let vp_size = camera.logical_viewport_size().unwrap();
        match *projection {
            Projection::Perspective(ref p) => {
                delta *= Vec2::new(p.fov * p.aspect_ratio, p.fov) / vp_size;
                multiplier = delta_grab.grab_pos.map_or_else(
                    || cam_tfm.translation.distance(cam.focus.translation),
                    |hit| hit.distance(cam_tfm.translation),
                );
            }
            Projection::Orthographic(ref p) => {
                delta *= Vec2::new(p.area.width(), p.area.height()) / vp_size;
            }
        }

        let mut delta = Vec3::ZERO;
        delta += cam.target_focus.forward() * delta.y;
        delta += cam.target_focus.right() * -delta.x;
        cam.target_focus.translation += delta * multiplier;

        delta_grab.delta = Vec2::ZERO;
    }
}
