use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};
use euclid::{Box2D, Point2D};
use std::ops::Mul;
mod net_vertex;
mod state;
mod tools;

///
#[derive(Resource, Default)]
struct Schematic {
    active_tool: tools::ActiveTool,
    state: state::State,
}

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
struct ActiveWireSeg;

#[derive(Component)]
struct WireSeg;

#[derive(Component)]
struct CursorMarker;

/// cursor position
#[derive(Resource, Default)]
struct CursorWorldCoords(SSPoint);

/// PhantomData tag for schematic space
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldSpace;

/// minimum rectangle containing the visible area
#[derive(Resource, Default)]
struct VisibleWorldRect(Option<Box2D<f32, WorldSpace>>);

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<CustomMaterial>::default());
        app.add_systems(Startup, (setup, setup_camera));
        app.add_systems(Update, (main, camera_transform, cursor));
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material2d for CustomMaterial {
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes
            .add(bevy::math::primitives::Rectangle::new(0.1, 0.1))
            .into(),
        material: materials.add(ColorMaterial::from(Color::RED)),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    },));
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes
            .add(bevy::math::primitives::Rectangle::new(0.1, 0.1))
            .into(),
        material: materials.add(ColorMaterial::from(Color::RED)),
        transform: Transform::from_translation(Vec3::new(1., 1., 0.)),
        ..default()
    },));

    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes
            .add(bevy::math::primitives::Rectangle::new(0.1, 0.1))
            .into(),
        material: materials.add(ColorMaterial::from(Color::YELLOW)),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    }, CursorMarker));

    commands.init_resource::<Schematic>();
    commands.init_resource::<CursorWorldCoords>();
    commands.init_resource::<VisibleWorldRect>();
}

use tools::ActiveTool;

use crate::types::{CSPoint, Point, SSPoint};

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut schematic: ResMut<Schematic>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut commands: Commands,
    schematic_coords: ResMut<CursorWorldCoords>,
) {
    let mut new_tool: Option<ActiveTool> = None;
    match &mut schematic.active_tool {
        tools::ActiveTool::Idle => {
            if keys.just_released(KeyCode::KeyW) {
                new_tool = Some(ActiveTool::Wiring(Box::new(tools::Wiring { mesh: None })))
            }
        }
        tools::ActiveTool::Wiring(wiring) => {
            let coords = schematic_coords.0;
            match &wiring.mesh {
                None => {
                    if buttons.just_released(MouseButton::Left) {
                        let mesh = meshes.add(
                            Mesh::new(
                                PrimitiveTopology::LineList,
                                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                            )
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                vec![
                                    Vec3::from(Point::from(coords.cast().cast_unit())),
                                    Vec3::from(Point::from(coords.cast().cast_unit())),
                                ],
                            ),
                        );
                        let ent = commands
                            .spawn((MaterialMesh2dBundle {
                                mesh: mesh.clone().into(),
                                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                                material: materials.add(CustomMaterial {
                                    color: Color::WHITE,
                                }),
                                ..default()
                            },))
                            .id();
                        wiring.mesh = Some((coords, mesh, ent));
                    }
                }
                Some(aws) => {
                    if keys.just_released(KeyCode::Escape) {
                        commands.entity(aws.2).despawn();
                    } else if buttons.just_released(MouseButton::Left) {
                        // left click while a wire seg is being drawn
                        if coords == aws.0 {
                            // terminate current line seg
                            new_tool =
                                Some(ActiveTool::Wiring(Box::new(tools::Wiring { mesh: None })));
                        } else {
                            // persist current segment:
                            let mesh = meshes.add(
                                Mesh::new(
                                    PrimitiveTopology::LineList,
                                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                                )
                                .with_inserted_attribute(
                                    Mesh::ATTRIBUTE_POSITION,
                                    vec![
                                        Vec3::from(Point::from(aws.0.cast().cast_unit())),
                                        Vec3::from(Point::from(coords.cast().cast_unit())),
                                    ],
                                ),
                            );
                            commands.spawn((MaterialMesh2dBundle {
                                mesh: mesh.clone().into(),
                                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                                material: materials.add(CustomMaterial {
                                    color: Color::WHITE,
                                }),
                                ..default()
                            }, WireSeg));
                            // set up next aws:
                            let wire = meshes.get_mut(aws.1.clone()).unwrap();
                            wire.insert_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                vec![
                                    Vec3::from(Point::from(coords.cast().cast_unit())),
                                    Vec3::from(Point::from(coords.cast().cast_unit())),
                                ],
                            );
                            wiring.mesh = Some((coords, aws.1.clone(), aws.2));
                        }
                    } else {
                        // just mouse movement
                        let wire = meshes.get_mut(aws.1.clone()).unwrap();
                        wire.insert_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![
                                Vec3::from(Point::from(aws.0.cast().cast_unit())),
                                Vec3::from(Point::from(coords.cast().cast_unit())),
                            ],
                        );
                        if let Some(aabb) = wire.compute_aabb() {
                            commands.entity(aws.2).insert(aabb);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if keys.just_released(KeyCode::Escape) {
        new_tool = Some(ActiveTool::Idle)
    }
    if let Some(tool) = new_tool {
        schematic.active_tool = tool;
    }
}

fn cursor(
    mut schematic_coords: ResMut<CursorWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MyCameraMarker>>,
    // mut q_activewire: Query<&mut ActiveWireSeg>,
    // mut assets: ResMut<Assets<Mesh>>,
    mut q_cursor: Query<&mut Transform, With<CursorMarker>>,
) {
    if let Ok((camera, cam_transform)) = q_camera.get_single() {
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(cam_transform, cursor))
            {
                schematic_coords.0 = CSPoint::new(coords.x, coords.y).round().cast().cast_unit();
                q_cursor.single_mut().translation = Vec3::new(coords.x.round(), coords.y.round(), 0.0);
            }
        }
    }
}

fn window_to_world(
    mut visible_coords: ResMut<VisibleWorldRect>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MyCameraMarker>>,
) {
    if let Ok((camera, cam_transform)) = q_camera.get_single() {
        if let Ok(window) = q_window.get_single() {
            // 0  1
            // 2  3
            let corners = [
                Vec2::ZERO,
                Vec2::new(window.width(), 0.),
                Vec2::new(0., window.height()),
                Vec2::new(window.width(), window.height()),
            ];
            let bb = corners.iter().filter_map(|p| {
                camera
                    .viewport_to_world_2d(cam_transform, *p)
                    .map(|v| Point2D::new(v.x, v.y))
            });
            let b = Box2D::from_points(bb);
            visible_coords.0 = Some(b);
            return;
        }
    }
    visible_coords.0 = None // if theres any problem, assume camera doesnt see anything
}

fn setup_camera(mut commands: Commands) {
    let scale = 0.1;
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0., 0., 1.0).with_scale(Vec3 {
                x: scale,
                y: scale,
                z: scale,
            }),
            projection: OrthographicProjection::default(),
            ..default()
        },
        MyCameraMarker,
    ));
}

fn camera_transform(
    mb: Res<ButtonInput<MouseButton>>,
    mut mm: EventReader<CursorMoved>,
    mut mw: EventReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &MyCameraMarker)>,
) {
    if let Ok(mut cam) = camera.get_single_mut() {
        if mb.pressed(MouseButton::Middle) {
            let mut pan = Vec3::ZERO;
            for m in mm.read() {
                if let Some(d) = m.delta {
                    pan += Vec3::new(-d.x, d.y, 0.0);
                }
            }
            let t = cam.0.scale.mul(pan);
            cam.0.translation += t;
        }
        for mwe in mw.read() {
            cam.0.scale *= 1. - mwe.y / 10.
        }
    }
}
