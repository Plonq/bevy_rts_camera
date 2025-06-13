//! A basic scene with some "terrain" and "units"
//! no bevy_rts_camera feature "controller"
//! interface with RtsCamera Directly
//! RTS camera.

use bevy::prelude::*;

use bevy_rts_camera::{Ground, RtsCamera, RtsCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        // Add `Ground` component to any entity you want the camera to treat as ground.
        Ground,
    ));
    // Some "terrain"
    let terrain_material = materials.add(Color::srgb(0.8, 0.7, 0.6));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(15.0, 1.0, 5.0))),
        MeshMaterial3d(terrain_material.clone()),
        Transform::from_xyz(15.0, 0.5, -5.0),
        Ground,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 5.0, 15.0))),
        MeshMaterial3d(terrain_material.clone()),
        Transform::from_xyz(-15.0, 2.5, 0.0),
        Ground,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(12.5))),
        MeshMaterial3d(terrain_material.clone()),
        Transform::from_xyz(0.0, 0.0, -23.0),
        Ground,
    ));
    // Some generic units that are not part of the 'Ground' (ignored for height calculation)
    for x in -5..5 {
        for z in -5..5 {
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.25, 1.25))),
                MeshMaterial3d(terrain_material.clone()),
                Transform::from_xyz(x as f32 * 0.7, 0.75, z as f32 * 0.7),
            ));
        }
    }
    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            150.0f32.to_radians(),
            -40.0f32.to_radians(),
            0.0,
        )),
    ));
    // Camera
    commands.spawn((RtsCamera {
        min_angle: (20.0_f32).to_radians(),
        dynamic_angle: false,
        ..default()
    },));
}

fn update(mut rts_camera: Single<&mut RtsCamera>, time: Res<Time>) {
    // You can directly modify the target with target_foucs
    rts_camera.target_focus.rotate_y(1.0 * time.delta_secs());

    // Control zooming
    rts_camera.target_zoom = time.elapsed_secs().sin() / 2.0 + 0.5;

    rts_camera.target_angle = (time.elapsed_secs().sin() * 20.0 + 45.0).to_radians()
}
