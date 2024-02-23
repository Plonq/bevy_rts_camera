// todo: re-enable
// #![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_mod_raycast::prelude::{Raycast, RaycastSettings};

/// Bevy plugin that contains the systems for controlling `RtsCamera` components.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, RtsCamera};
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
        app.add_systems(
            Update,
            // todo: optimize
            (
                zoom,
                set_look_direction,
                lateral_movement,
                follow_ground,
                move_towards_target,
                debug,
            )
                .chain()
                .in_set(RtsCameraSystemSet),
        );
    }
}

/// Base system set to allow ordering of `RtsCamera`
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RtsCameraSystemSet;

// #[derive(Resource, Copy, Clone, Debug, Eq, PartialEq)]
// pub struct RtsCamState {
//
// }

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
///             Camera3dBundle {
///                 transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
///                 ..default()
///             },
///             RtsCamera::default(),
///         ));
///  }
/// ```
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct RtsCamera {
    // Config
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub speed: f32,
    pub target: Vec3,
    pub zoom: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub angle: f32,
    pub enabled: bool,
}

impl Default for RtsCamera {
    fn default() -> Self {
        RtsCamera {
            // todo: test compatibility with input manager
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            speed: 1.0,
            target: Vec3::ZERO,
            zoom: 0.0,
            height_min: 0.1,
            height_max: 5.0,
            angle: 30.0f32.to_radians(),
            enabled: true,
        }
    }
}

impl RtsCamera {
    fn height(&self) -> f32 {
        self.height_max.lerp(self.height_min, self.zoom)
    }

    fn dist_to_target_lateral(&self) -> f32 {
        self.height() * self.angle.tan()
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct RtsCameraEye;

fn zoom(
    mut rts_camera: Query<(&Transform, &mut RtsCamera)>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    for (rts_cam_tfm, mut rts_cam) in rts_camera.iter_mut() {
        let zoom_amount = mouse_wheel
            .read()
            .map(|event| match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y * 0.001,
            })
            .fold(0.0, |acc, val| acc + val);
        let new_zoom = (rts_cam.zoom + zoom_amount * 0.5).clamp(0.0, 1.0);
        let zoom_delta = new_zoom - rts_cam.zoom;
        rts_cam.zoom = new_zoom;

        let height_delta = (rts_cam.height_max - rts_cam.height_min).abs() * zoom_delta;
        let new_target =
            rts_cam.target + rts_cam_tfm.forward() * (height_delta * rts_cam.angle.tan());
        rts_cam.target = new_target;
    }
}

fn set_look_direction(
    rts_camera: Query<(&RtsCamera, &Children)>,
    mut rts_cam_eye: Query<&mut Transform, With<RtsCameraEye>>,
) {
    for (rts_cam, children) in rts_camera.iter() {
        for child in children {
            if let Ok(mut eye_tfm) = rts_cam_eye.get_mut(*child) {
                eye_tfm.rotation = Quat::from_rotation_x(rts_cam.angle - 90f32.to_radians());
            }
        }
    }
}

fn lateral_movement(
    mut rts_camera: Query<(&Transform, &mut RtsCamera)>,
    button_input: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    for (rts_cam_tfm, mut rts_cam) in rts_camera.iter_mut() {
        gizmos.ray(
            rts_cam_tfm.translation,
            Vec3::from(rts_cam_tfm.forward()),
            Color::AQUAMARINE,
        );

        let mut delta = Vec3::ZERO;

        if button_input.pressed(rts_cam.key_up) {
            delta += rts_cam_tfm.forward() * rts_cam.speed;
        }
        if button_input.pressed(rts_cam.key_down) {
            delta += rts_cam_tfm.back() * rts_cam.speed;
        }
        if button_input.pressed(rts_cam.key_left) {
            delta += rts_cam_tfm.left() * rts_cam.speed;
        }
        if button_input.pressed(rts_cam.key_right) {
            delta += rts_cam_tfm.right() * rts_cam.speed;
        }

        let new_target =
            rts_cam.target + delta.normalize_or_zero() * time.delta_seconds() * 2.0 * rts_cam.speed;
        rts_cam.target = new_target;
    }
}

fn follow_ground(
    mut rts_camera: Query<(&Transform, &mut RtsCamera)>,
    mut gizmos: Gizmos,
    mut raycast: Raycast,
) {
    for (rts_cam_tfm, mut rts_cam) in rts_camera.iter_mut() {
        let dist_to_target_lateral = rts_cam.dist_to_target_lateral();

        // todo: add more rays to smooth transition between sudden ground height changes???
        // Ray starting directly above where the camera is looking, pointing straight down
        let ray1 = Ray3d::new(
            rts_cam_tfm.translation + rts_cam_tfm.forward() * dist_to_target_lateral,
            Vec3::from(rts_cam_tfm.down()),
        );
        let hits1 = raycast.debug_cast_ray(ray1, &RaycastSettings::default(), &mut gizmos);
        let hit1 = hits1.first().map(|(_, hit)| hit);
        if let Some(hit1) = hit1 {
            rts_cam.target.y = hit1.position().y + rts_cam.height();
        }
    }
}

fn move_towards_target(mut rts_camera: Query<(&mut Transform, &RtsCamera)>) {
    for (mut rts_cam_tfm, rts_cam) in rts_camera.iter_mut() {
        rts_cam_tfm.translation = rts_cam_tfm.translation.lerp(rts_cam.target, 0.2);
    }
}

fn debug(rts_camera: Query<(&Transform, &RtsCamera)>, mut gizmos: Gizmos) {
    for (rts_cam_tfm, _) in rts_camera.iter() {
        gizmos.ray(
            rts_cam_tfm.translation,
            Vec3::from(rts_cam_tfm.back()),
            Color::BLUE,
        );
        gizmos.ray(
            rts_cam_tfm.translation,
            Vec3::from(rts_cam_tfm.up()),
            Color::GREEN,
        );
        gizmos.ray(
            rts_cam_tfm.translation,
            Vec3::from(rts_cam_tfm.right()),
            Color::RED,
        );
    }
}
