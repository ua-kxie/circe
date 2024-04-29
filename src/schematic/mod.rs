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
pub(crate) mod grid;
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
struct Curpos {
    opt_ssp: Option<SSPoint>,
    opt_vsp: Option<CSPoint>,
}

#[derive(Event)]
struct NewCurposSSP(Option<SSPoint>);

#[derive(Event)]
struct NewCurposVSP(Option<CSPoint>);

/// PhantomData tag for schematic space
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldSpace;

/// minimum rectangle containing the visible area
#[derive(Resource, Default)]
struct VisibleWorldRect(Option<Box2D<f32, WorldSpace>>);

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WireMaterial>::default());
        app.add_systems(Startup, (setup, setup_camera));
        app.add_systems(
            Update,
            (main, camera_transform, cursor_update, draw_curpos_ssp),
        );
        app.add_event::<NewCurposSSP>();
        app.add_event::<NewCurposVSP>();
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WireMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material2d for WireMaterial {
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        // descriptor.primitive.polygon_mode = PolygonMode::Point;
        // descriptor.primitive.topology = PrimitiveTopology::PointList;
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid_materials: ResMut<Assets<grid::GridMaterial>>,
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

    // grid
    commands.spawn(MaterialMeshBundle {
        mesh: meshes
            .add(
                Mesh::new(
                    PrimitiveTopology::PointList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        Vec3::from([2.0, 2.0, 0.0]), 
                        Vec3::from([-2.0, -2.0, 0.0]),
                        Vec3::from([2.0, -2.0, 0.0]),
                        ],
                ),
                // Mesh::from(Cuboid::default())
            )
            .into(),
        material: grid_materials.add(grid::GridMaterial{}),
        ..default()
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(bevy::math::primitives::Rectangle::new(0.1, 0.1))
                .into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        CursorMarker,
    ));

    commands.init_resource::<Schematic>();
    commands.init_resource::<Curpos>();
    commands.init_resource::<VisibleWorldRect>();
}

use tools::ActiveTool;

use crate::types::{CSPoint, Point, SSPoint};

fn wiring(
    keys: &Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    coords1: Option<SSPoint>,
    wiremesh: &mut Option<(SSPoint, Handle<Mesh>, Entity)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WireMaterial>>,
    mut commands: Commands,
) -> Option<ActiveTool> {
    let coords = coords1.unwrap_or_default();
    let mut new_tool = None;
    match wiremesh {
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
                        material: materials.add(WireMaterial {
                            color: Color::WHITE,
                        }),
                        ..default()
                    },))
                    .id();
                *wiremesh = Some((coords, mesh, ent));
            }
        }
        Some(aws) => {
            if keys.just_released(KeyCode::Escape) {
                commands.entity(aws.2).despawn();
            } else if buttons.just_released(MouseButton::Left) {
                // left click while a wire seg is being drawn
                if coords == aws.0 {
                    // terminate current line seg
                    new_tool = Some(ActiveTool::Wiring(Box::new(tools::Wiring { mesh: None })));
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
                    commands.spawn((
                        MaterialMesh2dBundle {
                            mesh: mesh.clone().into(),
                            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                            material: materials.add(WireMaterial {
                                color: Color::WHITE,
                            }),
                            ..default()
                        },
                        WireSeg,
                    ));
                    // set up next aws:
                    let wire = meshes.get_mut(aws.1.clone()).unwrap();
                    wire.insert_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        vec![
                            Vec3::from(Point::from(coords.cast().cast_unit())),
                            Vec3::from(Point::from(coords.cast().cast_unit())),
                        ],
                    );
                    *wiremesh = Some((coords, aws.1.clone(), aws.2));
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
    new_tool
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut schematic: ResMut<Schematic>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<WireMaterial>>,
    commands: Commands,
    schematic_coords: ResMut<Curpos>,
) {
    let mut new_tool: Option<ActiveTool> = None;
    match &mut schematic.active_tool {
        tools::ActiveTool::Idle => {
            if keys.just_released(KeyCode::KeyW) {
                new_tool = Some(ActiveTool::Wiring(Box::new(tools::Wiring { mesh: None })))
            }
        }
        tools::ActiveTool::Wiring(wiringc) => {
            new_tool = wiring(
                &keys,
                buttons,
                schematic_coords.opt_ssp,
                &mut wiringc.mesh,
                meshes,
                materials,
                commands,
            );
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

/// this function retrieves the cursor position and stores it for use,
/// sending out events for world position changed, or viewport position changed
fn cursor_update(
    mut curpos: ResMut<Curpos>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MyCameraMarker>>,
    mut e_curpos_ssp: EventWriter<NewCurposSSP>,
    mut e_curpos_vsp: EventWriter<NewCurposVSP>,
) {
    let mut new_curpos = Curpos {
        opt_ssp: None,
        opt_vsp: None,
    };
    if let Ok((camera, cam_transform)) = q_camera.get_single() {
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(cam_transform, cursor))
            {
                new_curpos = Curpos {
                    opt_vsp: Some(CSPoint::new(coords.x, coords.y)),
                    opt_ssp: Some(CSPoint::new(coords.x, coords.y).round().cast().cast_unit()),
                };
            }
        }
    }

    if curpos.opt_vsp != new_curpos.opt_vsp {
        e_curpos_vsp.send(NewCurposVSP(new_curpos.opt_vsp));

        // snapped cursor may only change if raw cursor changes
        if curpos.opt_ssp != new_curpos.opt_ssp {
            e_curpos_ssp.send(NewCurposSSP(new_curpos.opt_ssp));
        }
    }
    *curpos = new_curpos;
}

fn draw_curpos_ssp(
    mut e_new_curpos_ssp: EventReader<NewCurposSSP>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<CursorMarker>>,
) {
    if let Some(NewCurposSSP(last_e)) = e_new_curpos_ssp.read().last() {
        if let Some(curpos_ssp) = last_e {
            *q_cursor.single_mut().1 = Visibility::Visible;
            q_cursor.single_mut().0.translation =
                Vec3::new(curpos_ssp.x.into(), curpos_ssp.y.into(), 0.0);
        } else {
            *q_cursor.single_mut().1 = Visibility::Hidden;
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
        Camera3dBundle {
            transform: Transform::from_xyz(0., 0., 1.0).with_scale(Vec3 {
                x: scale,
                y: scale,
                z: scale,
            }),
            projection: Projection::Orthographic(OrthographicProjection::default()),
            ..default()
        },
        MyCameraMarker,
    ));
}

fn camera_transform(
    mb: Res<ButtonInput<MouseButton>>,
    mut mm: EventReader<CursorMoved>,
    mut mw: EventReader<MouseWheel>,
    mut camera: Query<(&Camera, &mut Transform, &GlobalTransform), With<MyCameraMarker>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    // mut q_cursor: Query<&mut Transform, With<CursorMarker>>,
) {
    if let Ok((cam, mut transform, gt)) = camera.get_single_mut() {
        if mb.pressed(MouseButton::Middle) {
            let mut pan = Vec3::ZERO;
            for m in mm.read() {
                if let Some(d) = m.delta {
                    pan += Vec3::new(-d.x, d.y, 0.0);
                }
            }
            let t = transform.scale.mul(pan);
            transform.translation += t;
        }

        let mut curpos = None;
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| cam.viewport_to_world_2d(gt, cursor))
            {
                curpos = cam.viewport_to_world_2d(gt, coords);
            }
        }
        for mwe in mw.read() {
            transform.scale *= 1. - mwe.y / 5.
        }
        let mut curpos1 = None;
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| cam.viewport_to_world_2d(gt, cursor))
            {
                curpos1 = cam.viewport_to_world_2d(gt, coords);
            }
        }
        if curpos1 != curpos {
            println!("{curpos1:?} {curpos:?}")
        }
    }
}
