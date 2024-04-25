//! Demonstrates how you would replace the built-in controller with a custom solution, for example
//! so you can use an input manager.

use bevy::prelude::*;

use bevy_rts_camera::{
    DeltaGrab, Ground, RtsCamera, RtsCameraControls, RtsCameraControlsInputPlugin,
    RtsCameraControlsPlugin, RtsCameraControlsSystemSet, RtsCameraPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        // This plugin is used to make things easier, however you could remove it and manually
        // modify the `RtsCamera.target_*` properties directly if you wish.
        .add_plugins(RtsCameraControlsPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            // Run this before the controls systems otherwise there'll be a 1 frame delay
            rotate_with_keyboard.before(RtsCameraControlsSystemSet),
        )
        .run();
}

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
    // Camera
    commands.spawn((
        Camera3dBundle::default(),
        RtsCamera::default(),
        RtsCameraControls {
            button_grab: Some(MouseButton::Right),
            ..default()
        },
    ));
}

fn rotate_with_keyboard(
    mut delta_rotate: ResMut<DeltaRotate>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut delta = 0.0;
    let speed = 30.0f32.to_radians(); // 30 degrees/second

    if key_input.pressed(KeyCode::ArrowLeft) {
        delta -= speed;
    }
    if key_input.pressed(KeyCode::ArrowRight) {
        delta += speed;
    }

    delta_rotate.delta = delta * time.delta_seconds();
}
