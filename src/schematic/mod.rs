use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use euclid::{Box2D, Point2D};
use std::ops::Mul;
mod grid;
mod net_vertex;
mod state;
mod tools;

const MINSCALE: Vec3 = Vec3::splat(0.0001);
const MAXSCALE: Vec3 = Vec3::splat(100.0);

#[derive(Component)]
struct SchematicCameraMarker;

#[derive(Component)]
struct CursorMarker;

#[derive(Component)]
struct BackgroundMarker;

#[derive(Event)]
struct NewCurposSSP(Option<SSPoint>);

#[derive(Event)]
struct NewCurposVSP(Option<CSPoint>);

#[derive(Event)]
struct NewVisibleCanvasAABB;

/// minimum rectangle containing the visible area
#[derive(Default)]
struct VisibleCanvasAABB(Option<Box2D<i32, SchematicSpace>>);

/// cursor position
#[derive(Default)]
struct Curpos {
    opt_ssp: Option<SSPoint>,
    opt_vsp: Option<CSPoint>,
}

/// schematic resources
#[derive(Resource, Default)]
struct SchematicRes {
    visible_canvas_aabb: VisibleCanvasAABB,
    cursor_position: Curpos,
}

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((grid::Grid, tools::ToolsPlugin));
        app.add_systems(Startup, (setup, setup_camera));
        app.add_systems(
            Update,
            (
                camera_transform,
                cursor_update,
                draw_curpos_ssp,
                visible_canvas_aabb,
            ),
        );
        app.add_event::<NewCurposSSP>();
        app.add_event::<NewCurposVSP>();
        app.add_event::<NewVisibleCanvasAABB>();

        app.init_state::<tools::SchematicToolState>();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<grid::GridMaterial>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes
                .add(bevy::math::primitives::Rectangle::new(0.1, 0.1))
                .into(),
            material: grid_materials.add(grid::GridMaterial {
                color: Color::YELLOW,
            }),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        CursorMarker,
    ));

    commands.init_resource::<SchematicRes>();
}

use crate::types::{CSPoint, CanvasSpace, SSPoint, SchematicSpace};

/// this function maps the viewport rect onto the canvas (aabb) and sends out events
fn visible_canvas_aabb(
    mut schematic_res: ResMut<SchematicRes>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCameraMarker>>,
    mut e_new_viewport: EventWriter<NewVisibleCanvasAABB>,
) {
    let mut new_canvas_aabb: Option<Box2D<i32, SchematicSpace>> = None;
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
            let b: Box2D<f32, CanvasSpace> = Box2D::from_points(bb);
            new_canvas_aabb = Some(b.round_out().cast().cast_unit());
        }
    }
    if new_canvas_aabb != schematic_res.visible_canvas_aabb.0 {
        schematic_res.visible_canvas_aabb.0 = new_canvas_aabb;
        e_new_viewport.send(NewVisibleCanvasAABB);
    }
}

/// this function retrieves the cursor position and stores it for use,
/// sending out events for world position changed, or viewport position changed
fn cursor_update(
    mut schematic_res: ResMut<SchematicRes>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<SchematicCameraMarker>>,
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

    if schematic_res.cursor_position.opt_vsp != new_curpos.opt_vsp {
        e_curpos_vsp.send(NewCurposVSP(new_curpos.opt_vsp));

        // snapped cursor may only change if raw cursor changes
        if schematic_res.cursor_position.opt_ssp != new_curpos.opt_ssp {
            e_curpos_ssp.send(NewCurposSSP(new_curpos.opt_ssp));
        }
    }
    schematic_res.cursor_position = new_curpos;
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

fn setup_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scale = 0.1;
    let cam = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 0., 1.0).with_scale(Vec3 {
                    x: scale,
                    y: scale,
                    z: scale,
                }),
                projection: Projection::Orthographic(OrthographicProjection::default()),
                ..default()
            },
            SchematicCameraMarker,
            InheritedVisibility::VISIBLE,
        ))
        .id();

    // background
    let bg = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Plane3d::new(*Direction3d::Z).mesh().size(1e6, 1e6)),
            material: materials.add(Color::GREEN),
            transform: Transform::default().with_translation(Direction3d::NEG_Z * 1e2),
            ..default()
        })
        .id();

    commands.entity(cam).push_children(&[bg]);
}

fn camera_transform(
    mb: Res<ButtonInput<MouseButton>>,
    mut mm: EventReader<CursorMoved>,
    mut mw: EventReader<MouseWheel>,
    mut camera: Query<(&Camera, &mut Transform, &GlobalTransform), With<SchematicCameraMarker>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
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
            transform.scale = (transform.scale * (1. - mwe.y / 5.)).clamp(MINSCALE, MAXSCALE);
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
