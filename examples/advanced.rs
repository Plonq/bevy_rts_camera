//! A more complex scene with a moving "unit" that demonstrates how to jump to a location or lock
//! onto a particular entity.

use std::f32::consts::TAU;

use bevy::prelude::*;

use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin, RtsCameraSystemSet};

fn main() {
    App::new()
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
    commands.spawn((
        Camera3dBundle::default(),
        RtsCamera {
            // Increase min height (decrease max zoom)
            // height_min: 10.0,
            // Increase max height (decrease min zoom)
            height_max: 50.0,
            // Change the angle of the camera to 35 degrees
            min_angle: 35.0f32.to_radians(),
            // Decrease smoothing
            smoothness: 0.1,
            // Change starting position
            target_focus: Transform::from_xyz(3.0, 0.0, -3.0),
            // Change starting zoom level
            target_zoom: 0.2,
            // Disable dynamic angle (angle of camera will stay at `min_angle`)
            // dynamic_angle: false,
            ..default()
        },
        RtsCameraControls {
            // Change pan controls to WASD
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            // Change rotate to right click
            button_rotate: MouseButton::Right,
            // Change the width of the area that triggers edge pan. 0.1 is 10% of the window height.
            edge_pan_width: 0.1,
            // Increase pan speed
            pan_speed: 25.0,
            ..default()
        },
    ));
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
    key_input: Res<ButtonInput<KeyCode>>,
    cube_q: Query<&Transform, With<Move>>,
    mut cam_q: Query<&mut RtsCamera>,
) {
    for cube in cube_q.iter() {
        for mut cam in cam_q.iter_mut() {
            if key_input.pressed(KeyCode::KeyL) {
                cam.target_focus.translation = cube.translation;
                cam.snap = true;
            }
            if key_input.just_pressed(KeyCode::KeyK) {
                cam.target_focus.translation = cube.translation;
                cam.target_zoom = 0.4;
            }
        }
    }
}

fn toggle_controls(
    mut controls_q: Query<&mut RtsCameraControls>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    for mut controls in controls_q.iter_mut() {
        if key_input.just_pressed(KeyCode::KeyT) {
            controls.enabled = !controls.enabled;
        }
    }
}
