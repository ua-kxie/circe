use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};
use euclid::Box2D;

use crate::types::SchematicSpace;

use super::{
    ui::{self, GridMaterial}, NewVisibleCanvasAABB, SchematicRes
};

pub struct Grid;

impl Plugin for Grid {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GridMaterial>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, (major_grid_main, minor_grid_main));
    }
}

#[derive(Component)]
struct MinorGridMarker;

#[derive(Component)]
struct MajorGridMarker;

#[derive(Bundle)]
struct MinorGridBundle {
    mesh: MaterialMeshBundle<ui::GridMaterial>,
    grid: MinorGridMarker,
}

#[derive(Bundle)]
struct MajorGridBundle {
    mesh: MaterialMeshBundle<ui::GridMaterial>,
    grid: MajorGridMarker,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<ui::GridMaterial>>,
) {
    // grid
    commands.spawn(MinorGridBundle {
        mesh: MaterialMeshBundle {
            material: grid_materials.add(ui::GridMaterial {
                color: Color::rgba(0.5, 0.5, 0.5, 0.5),
            }),
            mesh: meshes
                .add(Mesh::new(
                    PrimitiveTopology::PointList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                ))
                .into(),
            ..default()
        },
        grid: MinorGridMarker,
    });

    // grid
    commands.spawn(MajorGridBundle {
        mesh: MaterialMeshBundle {
            material: grid_materials.add(ui::GridMaterial {
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            }),
            mesh: meshes
                .add(Mesh::new(
                    PrimitiveTopology::PointList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                ))
                .into(),
            ..default()
        },
        grid: MajorGridMarker,
    });

    // axis
    commands.spawn(
        MaterialMeshBundle {
            material: grid_materials.add(ui::GridMaterial {
                color: Color::WHITE,
            }),
            mesh: meshes
                .add(Mesh::new(
                    PrimitiveTopology::LineList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                ).with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        Vec3::new(-1.0, 0.0, 0.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, -1.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                    ],
                ))
                .into(),
            ..default()
        });
}

// place grid dots according to visible canvas aabb
fn minor_grid_main(
    mut g: Query<(Entity, &mut Handle<Mesh>), With<MinorGridMarker>>,
    mut e_new_viewport: EventReader<NewVisibleCanvasAABB>,
    schematic_res: ResMut<SchematicRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if let Some(_) = e_new_viewport.read().last() {
        let aabb = schematic_res.visible_canvas_aabb.0.unwrap();

        let grid = g.get_single_mut().unwrap();
        let gridmesh = meshes.get_mut(grid.1.id()).unwrap();
        gridmesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
        gridmesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, gridvec(aabb, 2, 1.0e4));
        if let Some(aabb) = gridmesh.compute_aabb() {
            commands.entity(grid.0).insert(aabb);
        }

    }
}

// place grid dots according to visible canvas aabb
fn major_grid_main(
    mut g: Query<(Entity, &mut Handle<Mesh>), With<MajorGridMarker>>,
    mut e_new_viewport: EventReader<NewVisibleCanvasAABB>,
    schematic_res: ResMut<SchematicRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if let Some(_) = e_new_viewport.read().last() {
        let aabb = schematic_res.visible_canvas_aabb.0.unwrap();

        let grid = g.get_single_mut().unwrap();
        let gridmesh = meshes.get_mut(grid.1.id()).unwrap();
        gridmesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
        gridmesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, gridvec(aabb, 16, 1.0e6));
        if let Some(aabb) = gridmesh.compute_aabb() {
            commands.entity(grid.0).insert(aabb);
        }

    }
}

fn gridvec(
    aabb: Box2D<i32, SchematicSpace>,
    spacing: i32,
    area_limit: f32,
) -> Vec<Vec3> {
    let area = aabb.height() as f32 * aabb.width() as f32;
    if area > area_limit {
        return vec![];
    }
    let height = aabb.height() / spacing + 1;
    let width = aabb.width() / spacing + 1;
    let minpoint = aabb.min / spacing * spacing;
    let veclen = (width * height).try_into().unwrap();
    let mut gridvec = vec![Vec3::splat(0.0); veclen];
    for x in 0..width {
        for y in 0..height {
            gridvec[(x * height + y) as usize] =
                Vec3::from_array([(minpoint.x + (x * spacing)) as f32, (minpoint.y + (y * spacing)) as f32, 0.0])
        }
    }
    gridvec
}