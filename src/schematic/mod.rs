use bevy::{
    input::mouse::MouseWheel,
    math::vec3,
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
mod state;
mod tools;
mod net_vertex;

///
#[derive(Resource, Default)]
struct Schematic {
    active_tool: tools::ActiveTool,

    state: state::State,
}

///

#[derive(Component)]
struct ActiveWireSeg {
    mesh: Handle<Mesh>,
}

#[derive(Component)]
struct MyCameraMarker;

/// cursor position
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

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
        app.add_systems(Startup, (setup, setup1, setup_camera, cursor_to_world));
        app.add_systems(Update, (main, camera_transform));
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
) {
    commands.init_resource::<Schematic>();
}

use tools::ActiveTool;

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut schematic: ResMut<Schematic>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut commands: Commands,
    mut schematic_coords: ResMut<CursorWorldCoords>,
) {
    let mut new_tool: Option<ActiveTool> = None;
    match &mut schematic.active_tool {
        tools::ActiveTool::Idle => {
            if keys.just_released(KeyCode::KeyW) {
                new_tool = Some(ActiveTool::Wiring(Box::new(tools::Wiring{mesh:None})))
            }
        }
        tools::ActiveTool::Wiring(wiring) => {
            let coords = schematic_coords.get_single();
            if buttons.just_released(MouseButton::Left) {
                match &wiring.mesh {
                    None => {
                        let mesh = meshes.add(
                            Mesh::new(
                                PrimitiveTopology::LineList,
                                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                            )
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                vec![vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 0.0)],
                            ),
                        );
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: mesh.clone().into(),
                                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                                material: materials.add(CustomMaterial {
                                    color: Color::WHITE,
                                }),
                                visibility: Visibility::Hidden,
                                ..default()
                            },
                        ));
                        wiring.mesh = Some(mesh);
                    },
                    Some(mesh) => {
                        let wire = meshes.get_mut(mesh).unwrap();
                        wire.insert_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 0.0)],
                        )
                    },
                }
            }
        }
        _ => {

        }
    }
    if keys.just_released(KeyCode::Escape) {
        new_tool = Some(ActiveTool::Idle)
    }
    if let Some(tool) = new_tool {
        schematic.active_tool = tool;
    }
}

fn cursor_to_world(
    mut schematic_coords: ResMut<CursorWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MyCameraMarker>>,
    // mut q_activewire: Query<&mut ActiveWireSeg>,
    // mut assets: ResMut<Assets<Mesh>>,
) {
    if let Ok((camera, cam_transform)) = q_camera.get_single() {
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(cam_transform, cursor))
            {
                schematic_coords.0 = coords;
                // let aw = q_activewire.get_single_mut().unwrap();
                // if let Some(mesh) = assets.get_mut(aw.mesh.id()) {
                //     mesh.insert_attribute(
                //         Mesh::ATTRIBUTE_POSITION,
                //         vec![vec3(coords.x, coords.y, 0.0), vec3(0.0, 0.0, 0.0)],
                //     );
                // }
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
            // projection: OrthographicProjection::default(),
            ..default()
        },
        MyCameraMarker,
    ));
}

fn setup1(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Grid
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

    commands.init_resource::<CursorWorldCoords>();
    commands.init_resource::<VisibleWorldRect>();
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
