#![allow(clippy::too_many_arguments)]

use crate::{Ground, RtsCamera, RtsCameraSystemSet};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use std::f32::consts::PI;

pub struct RtsCameraControlsPlugin;

impl Plugin for RtsCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (zoom, pan, grab_pan, rotate).before(RtsCameraSystemSet),
        );
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
///         ));
///  }
/// ```
#[derive(Component, Debug, PartialEq, Clone)]
pub struct RtsCameraControls {
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
    /// The key that will rotate the camera left.
    /// Defaults to `KeyCode::KeyQ`.
    pub key_rotate_left: KeyCode,
    /// The key that will rotate the camera right.
    /// Defaults to `KeyCode::KeyE`.
    pub key_rotate_right: KeyCode,
    /// How fast the keys will rotate the camera.
    /// Defaults to `16.0`.
    pub key_rotate_speed: f32,
    /// Whether to lock the mouse cursor in place while rotating.
    /// Defaults to `false`.
    pub lock_on_rotate: bool,
    /// The mouse button used to 'drag pan' the camera.
    /// Defaults to `None`.
    pub button_drag: Option<MouseButton>,
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
}

impl Default for RtsCameraControls {
    fn default() -> Self {
        RtsCameraControls {
            key_up: KeyCode::ArrowUp,
            key_down: KeyCode::ArrowDown,
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
            button_rotate: MouseButton::Middle,
            key_rotate_left: KeyCode::KeyQ,
            key_rotate_right: KeyCode::KeyE,
            key_rotate_speed: 16.0,
            lock_on_rotate: false,
            button_drag: None,
            lock_on_drag: false,
            edge_pan_width: 0.05,
            pan_speed: 15.0,
            zoom_sensitivity: 1.0,
            enabled: true,
        }
    }
}

pub fn zoom(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
) {
    for (mut cam, cam_controls) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        let zoom_amount = mouse_wheel
            .read()
            .map(|message| match message.unit {
                MouseScrollUnit::Line => message.y,
                MouseScrollUnit::Pixel => message.y * 0.001,
            })
            .fold(0.0, |acc, val| acc + val);
        let new_zoom =
            (cam.target_zoom + zoom_amount * 0.5 * cam_controls.zoom_sensitivity).clamp(0.0, 1.0);
        cam.target_zoom = new_zoom;
    }
}

pub fn pan(
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time<Real>>,
) {
    for (mut cam, controller) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        if controller
            .button_drag
            .is_some_and(|btn| mouse_input.pressed(btn))
        {
            continue;
        }

        let mut delta = Vec3::ZERO;

        // Keyboard pan
        if button_input.pressed(controller.key_up) {
            delta += Vec3::from(cam.target_focus.forward())
        }
        if button_input.pressed(controller.key_down) {
            delta += Vec3::from(cam.target_focus.back())
        }
        if button_input.pressed(controller.key_left) {
            delta += Vec3::from(cam.target_focus.left())
        }
        if button_input.pressed(controller.key_right) {
            delta += Vec3::from(cam.target_focus.right())
        }

        // Edge pan
        if delta.length_squared() == 0.0 && !mouse_input.pressed(controller.button_rotate) {
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
        &RtsCameraControls,
        &Camera,
        &Projection,
    )>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut ray_cast: MeshRayCast,
    mut ray_hit: Local<Option<Vec3>>,
    ground_q: Query<Entity, With<Ground>>,
    primary_window: Single<&mut Window, With<PrimaryWindow>>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut previous_mouse_grab_mode: Local<CursorGrabMode>,
) {
    for (cam_tfm, cam_gtfm, mut cam, controller, camera, projection) in cam_q
        .iter_mut()
        .filter(|(_, _, _, ctrl, _, _)| ctrl.enabled)
    {
        let Some(drag_button) = controller.button_drag else {
            continue;
        };

        if mouse_button.just_pressed(drag_button) && controller.lock_on_drag {
            let Some(cursor_position) = primary_window.cursor_position() else {
                return;
            };

            *previous_mouse_grab_mode = primary_cursor_options.grab_mode;
            primary_cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_cursor_options.visible = false;

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

        if mouse_button.just_released(drag_button) {
            *ray_hit = None;

            primary_cursor_options.grab_mode = *previous_mouse_grab_mode;
            primary_cursor_options.visible = true;
        }

        if mouse_button.pressed(drag_button) {
            let mut mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();

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
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    primary_window: Single<&mut Window, With<PrimaryWindow>>,
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut previous_mouse_grab_mode: Local<CursorGrabMode>,
) {
    for (mut cam, controller) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        if mouse_input.just_pressed(controller.button_rotate) && controller.lock_on_rotate {
            *previous_mouse_grab_mode = primary_cursor_options.grab_mode;
            primary_cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_cursor_options.visible = false;
        }

        if mouse_input.pressed(controller.button_rotate) {
            let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
            // Adjust based on window size, so that moving mouse entire width of window
            // will be one half rotation (180 degrees)
            let delta_x = mouse_delta.x / primary_window.width() * PI;
            cam.target_focus.rotate_local_y(-delta_x);
        } else {
            let left = if keys.pressed(controller.key_rotate_left) {
                1.0
            } else {
                0.0
            };
            let right = if keys.pressed(controller.key_rotate_right) {
                1.0
            } else {
                0.0
            };

            let delta = right - left;
            if delta != 0.0 {
                cam.target_focus.rotate_local_y(
                    delta / primary_window.width() * PI * controller.key_rotate_speed,
                );
            }
        }

        if mouse_input.just_released(controller.button_rotate) {
            primary_cursor_options.grab_mode = *previous_mouse_grab_mode;
            primary_cursor_options.visible = true;
        }
    }
}
