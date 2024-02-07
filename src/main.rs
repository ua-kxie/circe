use std::ops::Mul;

use bevy::{input::mouse::MouseWheel, prelude::*, sprite::MaterialMesh2dBundle};

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
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    });
}

fn camera_transform(
    mb: Res<ButtonInput<MouseButton>>,
    mut mm: EventReader<CursorMoved>,
    mut mw: EventReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &MyCameraMarker)>
) {
    if let Ok(mut a) = camera.get_single_mut() {
        if mb.pressed(MouseButton::Middle) {
            let mut pan = Vec3::ZERO;
            for m in mm.read() {
                if let Some(d) = m.delta {
                    pan += Vec3::new(-d.x, d.y, 0.0);
                }
            }
            let t = a.0.scale.mul(pan);
            a.0.translation += t;
        }
        for mwe in mw.read() {
            a.0.scale *= 1. - mwe.y / 10.
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