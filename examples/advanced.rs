//! A more complex scene with a moving "unit" that demonstrates how to jump to a location or lock
//! onto a particular entity.

use std::f32::consts::TAU;

use bevy::prelude::*;

use bevy_rts_camera::{
    CameraConfig, CameraControls, CameraSnapTo, CameraTargetTransform, CameraTargetZoom, Ground,
    RtsCamera, RtsCameraPlugin, RtsCameraSystemSet,
};

fn main() {
    App::new()
        // Set starting position of the camera (note Y is ignored because it's controlled by zoom)
        .insert_resource(CameraTargetTransform(Transform::from_xyz(3.0, 30.0, 1.0)))
        // Set starting zoom to 50%
        .insert_resource(CameraTargetZoom(0.5))
        .insert_resource(CameraConfig {
            // Change the width of the area that triggers edge pan. 0.1 is 10% of the window height.
            edge_pan_width: 0.1,
            // Increase pan speed
            pan_speed: 25.0,
            // Increase min height (decrease max zoom)
            height_min: 10.0,
            // Increase max height (decrease min zoom)
            height_max: 50.0,
            // Change the angle of the camera to 10 degrees (0 is looking straight down)
            angle: 10.0f32.to_radians(),
            // Decrease smoothing
            smoothness: 0.1,
        })
        .insert_resource(CameraControls {
            // Change pan controls to WASD
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            // Change rotate to right click
            button_rotate: MouseButton::Right,
            // Keep it enabled (wouldn't be much of a demo if we set this to false)
            enabled: true,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_unit, (lock_or_jump, toggle_controls))
                .chain()
                .before(RtsCameraSystemSet),
        )
        .run();
}

#[derive(Component)]
struct Move;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(80.0, 80.0)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        // Add `Ground` component to any entity you want the camera to treat as ground.
        Ground,
    ));
    // Some "terrain"
    let terrain_material = materials.add(Color::rgb(0.8, 0.7, 0.6));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(15.0, 1.0, 5.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(15.0, 0.5, -5.0),
            ..default()
        },
        Ground,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(10.0, 5.0, 15.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(-15.0, 2.5, 0.0),
            ..default()
        },
        Ground,
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(12.5)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -23.0),
            ..default()
        },
        Ground,
    ));
    // Some generic units that are not part of the 'Ground' (ignored for height calculation)
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Capsule3d::new(0.25, 1.25)),
                material: terrain_material.clone(),
                transform: Transform::from_xyz(x as f32 * 0.7, 0.75, z as f32 * 0.7),
                ..default()
            });
        }
    }
    // A moving unit that can be locked onto
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.25, 1.25)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(0.0, 0.75, 0.0),
            ..default()
        })
        .insert(Move);
    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            150.0f32.to_radians(),
            -40.0f32.to_radians(),
            0.0,
        )),
        ..default()
    });
    // Help text
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "\
Press K to jump to the moving unit
Hold L to lock onto the moving unit
Press T to toggle controls (K and L will still work)"
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        ..default()
    });
    // Camera
    commands.spawn((Camera3dBundle::default(), RtsCamera));
}

// Move a unit in a circle
fn move_unit(
    time: Res<Time>,
    mut cube_q: Query<&mut Transform, With<Move>>,
    mut angle: Local<f32>,
) {
    if let Ok(mut cube_tfm) = cube_q.get_single_mut() {
        // Rotate 20 degrees a second, wrapping around to 0 after a full rotation
        *angle += 20f32.to_radians() * time.delta_seconds() % TAU;
        // Convert angle to position
        let pos = Vec3::new(angle.sin() * 7.5, 0.75, angle.cos() * 7.5);
        cube_tfm.translation = pos;
    }
}

// Either jump to the moving unit (press K) or lock onto it (hold L)
fn lock_or_jump(
    mut commands: Commands,
    target_tfm: Res<CameraTargetTransform>,
    key_input: Res<ButtonInput<KeyCode>>,
    cube_q: Query<&Transform, With<Move>>,
) {
    for cube in cube_q.iter() {
        let new_tfm = target_tfm.0.with_translation(cube.translation);
        if key_input.pressed(KeyCode::KeyL) {
            commands.insert_resource(CameraTargetTransform(new_tfm));
            commands.insert_resource(CameraSnapTo(true));
        }
        if key_input.just_pressed(KeyCode::KeyK) {
            commands.insert_resource(CameraTargetTransform(new_tfm));
            // Zoom won't be 'locked' (it will still be smoothed), because zooming while locking
            // the position is possible
            commands.insert_resource(CameraTargetZoom(0.4));
        }
    }
}

fn toggle_controls(
    mut camera_controls: ResMut<CameraControls>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    if key_input.just_pressed(KeyCode::KeyT) {
        camera_controls.enabled = !camera_controls.enabled;
    }
}
