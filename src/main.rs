use std::ops::Mul;

use bevy::{input::mouse::MouseMotion, prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
struct MyCameraMarker;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
        MyCameraMarker,
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.)).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    });
}

fn camera_transform(
    mb: Res<ButtonInput<MouseButton>>,
    mut mm: EventReader<MouseMotion>,
    mut camera: Query<(&mut Transform, &MyCameraMarker)>
) {
    if mb.pressed(MouseButton::Middle) {
        if let Ok(mut a) = camera.get_single_mut() {
            let mut pan = Vec3::ZERO;
            for m in mm.read() {
                pan += Vec3::new(-m.delta.x, m.delta.y, 0.0);
            }
            let t = a.0.scale.mul(pan);
            a.0.translation += t;
        }
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup))
        .add_systems(Update, camera_transform)
        .run();
}