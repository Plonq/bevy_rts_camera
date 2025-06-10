#![allow(clippy::too_many_arguments)]

use crate::{Ground, RtsCamera, RtsCameraSystemSet};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use leafwing_input_manager::prelude::*;
use std::f32::consts::PI;

pub struct RtsCameraControlsPlugin;

impl Plugin for RtsCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<RtsCameraAction>::default())
            .add_systems(
                Update,
                (zoom, pan, grab_pan, rotate).before(RtsCameraSystemSet),
            );
    }
}

/// Define all possible action that a RtsCamera can perform.
///
/// # used [leafwing-input-manager](https://github.com/Leafwing-Studios/leafwing-input-manager/tree/main)
///
/// ## Modes
/// There are three mode for the RtsCamera
/// - ### Normal Mode:
///     In normal mode, the user can use:
///     - `Pan` action to move the camera target on the XZ plane
///     - `ZoomAxis` action to zoom near and far to the camera target
///     - `Rotate(bool)` action to rotate orbit around the target.(`bool` specify rotation direction)
/// - ### Rotate Mode:
///     In rotate mode, user can send an `Axislike` `RotationAxis` to rotate the camera around target,
///     this is used for taking mouse action, but it can also use other axis input.
///     
///     User can enter this mode by `RotateMode` action, a typical usage of this mode should be,
///     use a certain key to enter rotate mode, and some other axis to rotate
///     ```rust
///     InputMap::default()
///        .with(RtsCameraAction::RotateMode, MouseButton::Right)
///        .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
///     ```
/// - ### Grab Mode:
///     Like in rotate mode, in Grab Mode, user can send an `DualAxislike` `GrabAxis` to grab move the camera,
///     this is used for taking mouse action, but it can also use other axis input.
///     
///     User can enter this mode by `GrabMode` action, a typical usage of this mode should be,
///     use a certain key to enter grab mode, and some other axis to rotate
///     ```rust
///     InputMap::default()
///         .with(RtsCameraAction::GrabMode, MouseButton::Middle)
///         .with_dual_axis(RtsCameraAction::GrabAxis, MouseMove::default())
///     ```
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum RtsCameraAction {
    /// `DualAxisLike` action for XZ plane movement
    #[actionlike(DualAxis)]
    Pan,
    /// `AxisLike` action for zooming
    #[actionlike(Axis)]
    ZoomAxis,

    // Rotate
    /// `ButtonLike` action for entering rotate mode
    RotateMode,
    /// `AxisLike` action for rotating around the camera's target in rotation mode
    #[actionlike(Axis)]
    RotateAxis,
    /// `ButtonLike` action for rotating around  the camera's origin
    Rotate(bool),

    // Grab
    /// `ButtonLike` action for entering grab mode
    GrabMode,
    /// `DualAxisLike` action for moving camera by grabbing point in ground
    #[actionlike(DualAxis)]
    GrabAxis,
}

impl RtsCameraAction {
    /// A minimal input map
    /// ```rust
    /// InputMap::default()
    ///     // Pan Action
    ///     .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::arrow_keys())
    ///     // Zoom Action
    ///     .with_axis(RtsCameraAction::ZoomAxis, MouseScrollAxis::Y)
    ///     // Rotate
    ///     .with(RtsCameraAction::RotateMode, MouseButton::Right)
    ///     .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
    /// ```
    pub fn minimal_input_map() -> InputMap<RtsCameraAction> {
        InputMap::default()
            // Pan Action
            .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::arrow_keys())
            // Zoom Action
            .with_axis(RtsCameraAction::ZoomAxis, MouseScrollAxis::Y)
            // Rotate
            .with(RtsCameraAction::RotateMode, MouseButton::Right)
            .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
    }
    /// A fully featured input map
    /// ```rust
    /// InputMap::default()
    ///     // Pan Action
    ///     .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::wasd())
    ///     .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::arrow_keys())
    ///     .with_dual_axis(RtsCameraAction::Pan, VirtualDPad::action_pad())
    ///     // Zoom Action
    ///     .with_axis(
    ///         RtsCameraAction::ZoomAxis,
    ///         VirtualAxis::new(KeyCode::KeyE, KeyCode::KeyQ),
    ///     )
    ///     .with_axis(RtsCameraAction::ZoomAxis, MouseScrollAxis::Y)
    ///     // Rotate
    ///     .with(RtsCameraAction::RotateMode, MouseButton::Right)
    ///     .with_axis(RtsCameraAction::RotateAxis, MouseMoveAxis::X)
    ///     .with(RtsCameraAction::Rotate(true), KeyCode::KeyR)
    ///     .with(RtsCameraAction::Rotate(false), KeyCode::KeyF)
    ///     // Grab
    ///     .with(RtsCameraAction::GrabMode, MouseButton::Middle)
    ///     .with_dual_axis(RtsCameraAction::GrabAxis, MouseMove::default())
    /// ```
    pub fn full_input_map() -> InputMap<RtsCameraAction> {
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
    }
}

/// Optional camera controller. If you want to use an input manager, don't use this and instead
/// control the camera yourself by updating `RtsCamera.target_focus` and `RtsCamera.target_zoom`.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, RtsCamera, RtsCameraControls};
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
///             RtsCameraControls::default(),
///             RtsCameraAction::minimal_input_map(),
///         ));
///  }
/// ```
#[derive(Component, Debug, PartialEq, Clone)]
pub struct RtsCameraControls {
    /// How fast the keys will rotate the camera.
    /// Defaults to `16.0`.
    pub key_rotate_speed: f32,
    /// Whether to lock the mouse cursor in place while rotating.
    /// Defaults to `false`.
    pub lock_on_rotate: bool,
    /// Whether to lock the mouse cursor in place while dragging.
    /// Defaults to `false`.
    pub lock_on_drag: bool,
    /// How far away from the side of the screen edge pan will kick in, defined as a percentage
    /// of the window's height. Set to `0.0` to disable edge panning.
    /// Defaults to `0.05` (5%).
    pub edge_pan_width: f32,
    /// Speed of camera pan (either via keyboard controls or edge panning).
    /// Defaults to `15.0`.
    pub pan_speed: f32,
    /// How much the camera will zoom.
    /// Defaults to `1.0`.
    pub zoom_sensitivity: f32,
    /// Whether these controls are enabled.
    /// Defaults to `true`.
    pub enabled: bool,
    /// Whether the controller is in rotate mode.
    /// This is use to make sure rotate mode and grab mode don't collide
    pub rotate_mode: bool,
    /// Whether the controller is in grab mode
    /// This is use to make sure rotate mode and grab mode don't collide
    pub grab_mode: bool,
}

impl Default for RtsCameraControls {
    fn default() -> Self {
        RtsCameraControls {
            key_rotate_speed: 16.0,
            lock_on_rotate: false,
            lock_on_drag: false,
            edge_pan_width: 0.05,
            pan_speed: 15.0,
            zoom_sensitivity: 1.0,
            enabled: true,
            grab_mode: false,
            rotate_mode: false,
        }
    }
}

pub fn zoom(
    mut cam_q: Query<(
        &mut RtsCamera,
        &RtsCameraControls,
        &ActionState<RtsCameraAction>,
    )>,
) {
    for (mut cam, controller, action_state) in cam_q.iter_mut().filter(|(_, ctrl, _)| ctrl.enabled)
    {
        let Some(axis_data) = action_state.axis_data(&RtsCameraAction::ZoomAxis) else {
            continue;
        };
        let zoom_amount = axis_data.value;
        let new_zoom =
            (cam.target_zoom + zoom_amount * 0.5 * controller.zoom_sensitivity).clamp(0.0, 1.0);
        cam.target_zoom = new_zoom;
    }
}

pub fn pan(
    mut cam_q: Query<(
        &mut RtsCamera,
        &RtsCameraControls,
        &ActionState<RtsCameraAction>,
    )>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time<Real>>,
) {
    for (mut cam, controller, action_state) in cam_q.iter_mut().filter(|(_, ctrl, _)| ctrl.enabled)
    {
        if action_state.pressed(&RtsCameraAction::GrabMode)
            | action_state.pressed(&RtsCameraAction::RotateMode)
            | controller.grab_mode
            | controller.rotate_mode
        {
            continue;
        }

        let mut delta = Vec3::ZERO;

        if let Some(pan) = action_state.dual_axis_data(&RtsCameraAction::Pan) {
            delta = Vec3::from(cam.target_focus.forward()) * pan.pair.y
                + Vec3::from(cam.target_focus.left()) * -pan.pair.x;
        }

        if delta.length_squared() == 0.0 {
            // // Edge pan
            if let Ok(primary_window) = primary_window_q.single() {
                if let Some(cursor_position) = primary_window.cursor_position() {
                    let win_w = primary_window.width();
                    let win_h = primary_window.height();
                    let pan_width = win_h * controller.edge_pan_width;
                    // Pan left
                    if cursor_position.x < pan_width {
                        delta += Vec3::from(cam.target_focus.left())
                    }
                    // Pan right
                    if cursor_position.x > win_w - pan_width {
                        delta += Vec3::from(cam.target_focus.right())
                    }
                    // Pan up
                    if cursor_position.y < pan_width {
                        delta += Vec3::from(cam.target_focus.forward())
                    }
                    // Pan down
                    if cursor_position.y > win_h - pan_width {
                        delta += Vec3::from(cam.target_focus.back())
                    }
                }
            }
        }

        let new_target = cam.target_focus.translation
            + delta.normalize_or_zero()
            * time.delta_secs()
            * controller.pan_speed
            // Scale based on zoom so it (roughly) feels the same speed at different zoom levels
            * cam.target_zoom.remap(0.0, 1.0, 1.0, 0.5);
        cam.target_focus.translation = new_target;
    }
}

pub fn grab_pan(
    mut cam_q: Query<(
        &Transform,
        &GlobalTransform,
        &mut RtsCamera,
        &mut RtsCameraControls,
        &Camera,
        &Projection,
        &ActionState<RtsCameraAction>,
    )>,
    mut ray_cast: MeshRayCast,
    mut ray_hit: Local<Option<Vec3>>,
    ground_q: Query<Entity, With<Ground>>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut previous_mouse_grab_mode: Local<CursorGrabMode>,
) {
    for (cam_tfm, cam_gtfm, mut cam, mut controller, camera, projection, action_state) in cam_q
        .iter_mut()
        .filter(|(_, _, _, ctrl, _, _, _)| ctrl.enabled)
    {
        if controller.rotate_mode {
            continue;
        }

        let Ok(mut primary_window) = primary_window_q.single_mut() else {
            return;
        };

        if action_state.just_pressed(&RtsCameraAction::GrabMode) {
            controller.grab_mode = true;

            let Some(cursor_position) = primary_window.cursor_position() else {
                return;
            };

            if controller.lock_on_drag {
                *previous_mouse_grab_mode = primary_window.cursor_options.grab_mode;
                primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor_options.visible = false;
            }

            if let Ok(cursor_ray) = camera.viewport_to_world(cam_gtfm, cursor_position) {
                *ray_hit = ray_cast
                    .cast_ray(
                        cursor_ray,
                        &MeshRayCastSettings {
                            filter: &|entity| ground_q.get(entity).is_ok(),
                            ..default()
                        },
                    )
                    .first()
                    .map(|(_, hit)| hit.point);
            }
        }

        if action_state.just_released(&RtsCameraAction::GrabMode) {
            controller.grab_mode = false;

            *ray_hit = None;

            if controller.lock_on_drag {
                primary_window.cursor_options.grab_mode = *previous_mouse_grab_mode;
                primary_window.cursor_options.visible = true;
            }
        }

        if action_state.pressed(&RtsCameraAction::GrabMode) {
            let mut mouse_delta = action_state.axis_pair(&RtsCameraAction::GrabAxis);

            let mut multiplier = 1.0;
            let vp_size = camera.logical_viewport_size().unwrap();
            match *projection {
                Projection::Perspective(ref p) => {
                    mouse_delta *= Vec2::new(p.fov * p.aspect_ratio, p.fov) / vp_size;
                    multiplier = (*ray_hit).map_or_else(
                        || cam_tfm.translation.distance(cam.focus.translation),
                        |hit| hit.distance(cam_tfm.translation),
                    );
                }
                Projection::Orthographic(ref p) => {
                    mouse_delta *= Vec2::new(p.area.width(), p.area.height()) / vp_size;
                }
                Projection::Custom(ref _p) => todo!(),
            }

            let mut delta = Vec3::ZERO;
            delta += cam.target_focus.forward() * mouse_delta.y;
            delta += cam.target_focus.right() * -mouse_delta.x;
            cam.target_focus.translation += delta * multiplier;
        }
    }
}

pub fn rotate(
    mut cam_q: Query<(
        &mut RtsCamera,
        &mut RtsCameraControls,
        &ActionState<RtsCameraAction>,
    )>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut previous_mouse_grab_mode: Local<CursorGrabMode>,
) {
    if let Ok(mut primary_window) = primary_window_q.single_mut() {
        for (mut cam, mut controller, action_state) in
            cam_q.iter_mut().filter(|(_, ctrl, _)| ctrl.enabled)
        {
            if controller.grab_mode {
                continue;
            }

            if action_state.just_pressed(&RtsCameraAction::RotateMode) {
                controller.rotate_mode = true;

                if controller.lock_on_rotate {
                    *previous_mouse_grab_mode = primary_window.cursor_options.grab_mode;
                    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    primary_window.cursor_options.visible = false;
                }
            }

            if action_state.pressed(&RtsCameraAction::RotateMode) {
                let Some(axis_data) = action_state.axis_data(&RtsCameraAction::RotateAxis) else {
                    continue;
                };
                let value = axis_data.value / primary_window.width() * PI;
                cam.target_focus.rotate_local_y(value);
            } else if action_state.pressed(&RtsCameraAction::Rotate(true)) {
                cam.target_focus.rotate_local_y(
                    1.0 / primary_window.width() * PI * controller.key_rotate_speed,
                );
            } else if action_state.pressed(&RtsCameraAction::Rotate(false)) {
                cam.target_focus.rotate_local_y(
                    -1.0 / primary_window.width() * PI * controller.key_rotate_speed,
                );
            }

            if action_state.just_released(&RtsCameraAction::RotateMode) {
                controller.rotate_mode = false;

                if controller.lock_on_rotate {
                    primary_window.cursor_options.grab_mode = *previous_mouse_grab_mode;
                    primary_window.cursor_options.visible = true;
                }
            }
        }
    }
}
