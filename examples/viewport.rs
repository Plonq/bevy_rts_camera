//! The basic scene showcasing restricting edge panning to the camera's viewport

use bevy::{camera::Viewport, math::VectorSpace, prelude::*, window::PrimaryWindow};

use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin};

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
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
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

    let window = primary_window_q.single().unwrap();
    // Setup 2 cameras each taking up half the window in width
    // Camera 1
    commands.spawn((
        RtsCamera::default(),
        RtsCameraControls {
            //Restricts edge panning to the viewport instead of the window
            edge_pan_restrict_to_viewport: true,
            ..default()
        },
        Camera {
            order: 0,
            viewport: Some(Viewport {
                // Start the viewport in the top left
                physical_position: uvec2(0, 0),
                // Have it take up half of the window width
                physical_size: uvec2(window.physical_width() / 2, window.physical_height()),
                ..default()
            }),
            ..default()
        },
    ));
    // Camera 2
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 1,
            viewport: Some(Viewport {
                // Start the viewport in the top middle
                physical_position: uvec2(window.physical_width() / 2, 0),
                // Have it take up the other half of the window
                physical_size: uvec2(window.physical_width() / 2, window.physical_height()),
                ..default()
            }),
            ..default()
        },
    ));
}
