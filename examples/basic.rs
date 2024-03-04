//! A basic scene with some "terrain" and "units" to demonstrate how to set up a basic
//! RTS camera.

use bevy::prelude::*;

use bevy_rts_camera::{Ground, RtsCamera, RtsCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(15.0, 15.0)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        })
        // Add `Ground` component to any mesh you want the camera to treat as ground.
        // todo: scale of everything?
        .insert(Ground);
    // Some "terrain"
    let terrain_material = materials.add(Color::rgb(0.8, 0.7, 0.6));
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(3.0, 0.2, 1.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(3.0, 0.1, -1.0),
            ..default()
        })
        .insert(Ground);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 1.0, 3.0)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(-3.0, 0.5, 0.0),
            ..default()
        })
        .insert(Ground);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Sphere::new(2.5)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -4.2),
            ..default()
        })
        .insert(Ground);
    // Some generic units that are not part of the 'Ground' (ignored for height calculation)
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Capsule3d::new(0.08, 0.22)),
                material: terrain_material.clone(),
                transform: Transform::from_xyz(x as f32 / 5.0, 0.19, z as f32 / 5.0),
                ..default()
            });
        }
    }
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // Camera
    commands.spawn((Camera3dBundle::default(), RtsCamera));
}
