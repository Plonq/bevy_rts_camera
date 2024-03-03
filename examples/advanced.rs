//! A more complex scene with a moving "unit" that demonstrates how to jump to a location or lock
//! onto a particular entity.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use bevy_rts_camera::{
    CameraSnapTo, CameraTargetTransform, Ground, RtsCamera, RtsCameraPlugin, RtsCameraSystemSet,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RtsCameraPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_unit, lock_or_jump).chain().before(RtsCameraSystemSet),
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
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(15.0, 15.0)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        })
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
    // A moving unit that can be locked onto
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.1, 0.3)),
            material: terrain_material.clone(),
            transform: Transform::from_xyz(0.0, 0.25, 0.0),
            ..default()
        })
        .insert(Move);
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
    // Help text
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Press K to jump to the moving unit\nHold L to lock onto the moving unit"
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        ..default()
    });
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
        let pos = Vec3::new(angle.sin() * 1.5, 0.25, angle.cos() * 1.5);
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
            commands.insert_resource(CameraTargetTransform(new_tfm))
        }
    }
}
