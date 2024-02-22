//! Demonstrates the simplest usage

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rts_camera::{RtsCamera, RtsCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, swap_cameras)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(15.0, 15.0)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });
    // Some "terrain"
    let terrain_material = materials.add(Color::rgb(0.8, 0.7, 0.6));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 0.5, 1.0)),
        material: terrain_material.clone(),
        transform: Transform::from_xyz(0.0, 0.25, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(3.0, 0.2, 1.0)),
        material: terrain_material.clone(),
        transform: Transform::from_xyz(3.0, 0.1, -1.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(2.0, 0.3, 3.0)),
        material: terrain_material.clone(),
        transform: Transform::from_xyz(-3.0, 0.15, 0.0),
        ..default()
    });
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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 5.0, 1.5))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RtsCamera::default(),
    ));
    // commands
    //     .spawn((
    //         // Camera3dBundle {
    //         //     transform: Transform::from_translation(Vec3::new(0.0, 2.5, 1.5))
    //         //         .looking_at(Vec3::ZERO, Vec3::Y),
    //         //     ..default()
    //         // },
    //         RtsCamera::default(),
    //         TransformBundle::default(),
    //     ))
    //     .with_children(|parent| {
    //         parent.spawn(Camera3dBundle::default());
    //     });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(8.0, 1.0, 14.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                is_active: false,
                ..default()
            },
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn swap_cameras(
    mut orbit_cam: Query<&mut Camera, With<PanOrbitCamera>>,
    mut rts_cam: Query<&mut Camera, (With<RtsCamera>, Without<PanOrbitCamera>)>,
    button_input: Res<ButtonInput<KeyCode>>,
) {
    if button_input.just_pressed(KeyCode::Space) {
        let mut orbit_cam = orbit_cam.get_single_mut().unwrap();
        let mut rts_cam = rts_cam.get_single_mut().unwrap();
        orbit_cam.is_active = !orbit_cam.is_active;
        rts_cam.is_active = !rts_cam.is_active;
    }
}
