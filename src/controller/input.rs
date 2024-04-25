use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_raycast::immediate::{Raycast, RaycastSettings};
use bevy_mod_raycast::CursorRay;
use std::f32::consts::PI;

use crate::controller::logic::DeltaZoom;
use crate::controller::{DeltaPan, DeltaRotate};
use crate::{DeltaGrab, Ground, RtsCamera};

/// Optional camera controller. If you want to use an input manager, don't use this and instead
/// control the camera yourself by updating `RtsCamera.target_focus` and `RtsCamera.target_zoom`.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, RtsCameraControlsPlugin, RtsCameraControlsInputPlugin, RtsCamera, RtsCameraControls};
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(DefaultPlugins)
/// #         .add_plugins(RtsCameraPlugin)
/// #         .add_plugins(RtsCameraControlsPlugin)
/// #         .add_plugins(RtsCameraControlsInputPlugin)
/// #         .add_systems(Startup, setup)
/// #         .run();
/// # }
/// fn setup(mut commands: Commands) {
///     commands
///         .spawn((
///             Camera3dBundle::default(),
///             RtsCamera::default(),
///             RtsCameraControls::default(),
///         ));
///  }
/// ```
#[derive(Component, Debug, PartialEq, Clone)]
pub struct RtsCameraControls {
    /// The key that will pan the camera forward.
    /// Defaults to `KeyCode::ArrowUp`.
    pub key_up: KeyCode,
    /// The key that will pan the camera backward.
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
    /// The mouse button used to 'drag pan' the camera.
    /// Defaults to `None`.
    pub button_grab: Option<MouseButton>,
    /// How far away from the side of the screen edge pan will kick in, defined as a percentage
    /// of the window's height. Set to `0.0` to disable edge panning.
    /// Defaults to `0.05` (5%).
    pub edge_pan_width: f32,
    /// Speed of camera pan (either via keyboard controls or edge panning).
    /// Defaults to `1.0`.
    pub pan_speed: f32,
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
            button_grab: None,
            edge_pan_width: 0.05,
            pan_speed: 15.0,
            enabled: true,
        }
    }
}

pub fn zoom(
    mut delta_zoom: ResMut<DeltaZoom>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cam_q: Query<&RtsCameraControls>,
) {
    for _ in cam_q.iter_mut().filter(|ctrl| ctrl.enabled) {
        let zoom_amount = mouse_wheel
            .read()
            .map(|event| match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y * 0.001,
            })
            .fold(0.0, |acc, val| acc + val);
        delta_zoom.delta = zoom_amount;
    }
}

pub fn pan(
    mut delta_pan: ResMut<DeltaPan>,
    cam_q: Query<&RtsCameraControls>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    for controller in cam_q.iter().filter(|ctrl| ctrl.enabled) {
        if controller
            .button_grab
            .is_some_and(|btn| mouse_input.pressed(btn))
        {
            continue;
        }

        let mut delta = Vec3::ZERO;

        // Keyboard pan
        if button_input.pressed(controller.key_up) {
            delta += Vec3::NEG_Z;
        }
        if button_input.pressed(controller.key_down) {
            delta += Vec3::Z;
        }
        if button_input.pressed(controller.key_left) {
            delta += Vec3::NEG_X;
        }
        if button_input.pressed(controller.key_right) {
            delta += Vec3::X;
        }

        // Edge pan
        if delta.length_squared() == 0.0 && !mouse_input.pressed(controller.button_rotate) {
            if let Ok(primary_window) = primary_window_q.get_single() {
                if let Some(cursor_position) = primary_window.cursor_position() {
                    let win_w = primary_window.width();
                    let win_h = primary_window.height();
                    let pan_width = win_h * controller.edge_pan_width;
                    // Pan left
                    if cursor_position.x < pan_width {
                        delta += Vec3::NEG_X;
                    }
                    // Pan right
                    if cursor_position.x > win_w - pan_width {
                        delta += Vec3::X
                    }
                    // Pan up
                    if cursor_position.y < pan_width {
                        delta += Vec3::NEG_Z;
                    }
                    // Pan down
                    if cursor_position.y > win_h - pan_width {
                        delta += Vec3::Z;
                    }
                }
            }
        }

        delta_pan.delta = delta.normalize_or_zero() * time.delta_seconds() * controller.pan_speed;
    }
}

pub fn grab_pan(
    mut delta_grab: ResMut<DeltaGrab>,
    mut cam_q: Query<(Entity, &RtsCameraControls)>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut raycast: Raycast,
    cursor_ray: Res<CursorRay>,
    ground_q: Query<Entity, With<Ground>>,
) {
    for (entity, controller) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        let Some(drag_button) = controller.button_grab else {
            continue;
        };

        if mouse_button.just_pressed(drag_button) {
            delta_grab.entity = Some(entity);

            if let Some(cursor_ray) = **cursor_ray {
                delta_grab.grab_pos = raycast
                    .cast_ray(
                        cursor_ray,
                        &RaycastSettings {
                            filter: &|entity| ground_q.get(entity).is_ok(),
                            ..default()
                        },
                    )
                    .first()
                    .map(|(_, hit)| hit.position());
            }
        }

        if mouse_button.just_released(drag_button) {
            delta_grab.grab_pos = None;
        }

        if mouse_button.pressed(drag_button) {
            delta_grab.delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
        }
    }
}

pub fn rotate(
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls, &Camera)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for (mut cam, controller, camera) in cam_q.iter_mut().filter(|(_, ctrl, _)| ctrl.enabled) {
        if mouse_input.pressed(controller.button_rotate) {
            let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
            if let Some(viewport_size) = camera.logical_viewport_size() {
                let delta_x = mouse_delta.x / viewport_size.x * PI;
                cam.target_focus.rotate_local_y(-delta_x);
            }
        }
    }
}
