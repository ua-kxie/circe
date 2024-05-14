use bevy::{input::mouse::MouseWheel, prelude::*, window::PrimaryWindow};
use euclid::{Box2D, Point2D};

mod grid;
mod net_vertex;
mod selectable;
mod state;
mod tools;

const MINSCALE: f32 = 0.001;
const MAXSCALE: f32 = 1.0;

#[derive(Component)]
struct InfoTextMarker;

#[derive(Component)]
struct SchematicCameraMarker;

#[derive(Component)]
struct CursorMarker;

#[derive(Component)]
struct BackgroundMarker;

#[derive(Event)]
struct NewCurposI(Option<IVec2>);

#[derive(Event)]
struct NewCurposF(Option<Vec2>);

#[derive(Event)]
struct NewVisibleCanvasAABB;

/// minimum rectangle containing the visible area
#[derive(Default)]
struct VisibleCanvasAABB(Option<Box2D<i32, SchematicSpace>>);

/// cursor position
#[derive(Default)]
struct Curpos {
    opt_ssp: Option<IVec2>,
    opt_vsp: Option<Vec2>,
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
        app.add_plugins((grid::Grid, tools::ToolsPlugin, selectable::SelectablePlugin));
        app.add_systems(Startup, (setup, setup_camera));
        app.add_systems(
            Update,
            (
                camera_transform,
                cursor_update,
                draw_curpos_ssp,
                visible_canvas_aabb,
                update_info_text,
            ),
        );
        app.add_event::<NewCurposI>();
        app.add_event::<NewCurposF>();
        app.add_event::<NewVisibleCanvasAABB>();

        app.init_state::<tools::SchematicToolState>();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<grid::GridMaterial>>,
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
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
        InfoTextMarker,
    ));

    commands.init_resource::<SchematicRes>();
}

fn update_info_text(
    mut text: Query<&mut Text, With<InfoTextMarker>>,
    projection: Query<&Projection, With<SchematicCameraMarker>>,
    schematic_res: Res<SchematicRes>,
) {
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;
    *text = "".to_string();

    if let Some(ssp) = schematic_res.cursor_position.opt_ssp {
        text.push_str(&format!("x: {:+03}; y: {:+03}; ", ssp.x, ssp.y))
    }
    if let Projection::Orthographic(opj) = projection.single() {
        text.push_str(&format!("scale: {:.4};", opj.scale));
    }
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
    mut e_curpos_ssp: EventWriter<NewCurposI>,
    mut e_curpos_vsp: EventWriter<NewCurposF>,
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
                    opt_vsp: Some(coords),
                    opt_ssp: Some(coords.round().as_ivec2()),
                };
            }
        }
    }

    if schematic_res.cursor_position.opt_vsp != new_curpos.opt_vsp {
        e_curpos_vsp.send(NewCurposF(new_curpos.opt_vsp));

        // snapped cursor may only change if raw cursor changes
        if schematic_res.cursor_position.opt_ssp != new_curpos.opt_ssp {
            e_curpos_ssp.send(NewCurposI(new_curpos.opt_ssp));
        }
    }
    schematic_res.cursor_position = new_curpos;
}

fn draw_curpos_ssp(
    mut e_new_snapped_curpos: EventReader<NewCurposI>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<CursorMarker>>,
) {
    if let Some(NewCurposI(opt_new_curpos)) = e_new_snapped_curpos.read().last() {
        if let Some(new_curpos) = opt_new_curpos {
            *q_cursor.single_mut().1 = Visibility::Visible;
            q_cursor.single_mut().0.translation = new_curpos.as_vec2().extend(0.0);
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
    let cam = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 0., 1.0),
                projection: Projection::Orthographic(OrthographicProjection {
                    scale: 0.1,
                    ..Default::default()
                }),
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
    mut camera: Query<
        (&Camera, &mut Transform, &GlobalTransform, &mut Projection),
        With<SchematicCameraMarker>,
    >,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok((cam, mut transform, gt, mut pj)) = camera.get_single_mut() {
        if mb.pressed(MouseButton::Middle) {
            if let Projection::Orthographic(opj) = &mut *pj {
                let mut pan = Vec3::ZERO;
                for m in mm.read() {
                    if let Some(d) = m.delta {
                        pan += Vec3::new(-d.x, d.y, 0.0);
                    }
                }
                let t = pan * opj.scale;
                transform.translation += t;
            }
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
            if let Projection::Orthographic(opj) = &mut *pj {
                opj.scale = (opj.scale * (1. - mwe.y / 5.)).clamp(MINSCALE, MAXSCALE);
            }
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
