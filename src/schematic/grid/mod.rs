use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};
use euclid::{Box2D, Point2D};

use crate::types::SchematicSpace;

use super::{
    ui::{self, GridMaterial},
    NewVisibleCanvasAABB, VisibleCanvasAABB,
};

pub struct Grid;

impl Plugin for Grid {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<GridMaterial>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, main);
    }
}

#[derive(Component)]
struct GridMarker;

#[derive(Bundle)]
struct GridBundle {
    mesh: MaterialMeshBundle<ui::GridMaterial>,
    grid: GridMarker,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grid_materials: ResMut<Assets<ui::GridMaterial>>,
) {
    // grid
    commands.spawn(GridBundle {
        mesh: MaterialMeshBundle {
            material: grid_materials.add(ui::GridMaterial {
                color: Color::WHITE,
            }),
            mesh: meshes
                .add(Mesh::new(
                    PrimitiveTopology::PointList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                ))
                .into(),
            ..default()
        },
        grid: GridMarker,
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
fn main(
    mut g: Query<(Entity, &mut Handle<Mesh>), With<GridMarker>>,
    mut e_new_viewport: EventReader<NewVisibleCanvasAABB>,
    visible_canvas_aabb: ResMut<VisibleCanvasAABB>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if let Some(_) = e_new_viewport.read().last() {
        let aabb = visible_canvas_aabb.0.unwrap();
        let grid = g.get_single_mut().unwrap();
        let gridmesh = meshes.get_mut(grid.1.id()).unwrap();

        // delete old grid
        gridmesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
        
        gridmesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, gridvec(aabb));

        if let Some(aabb) = gridmesh.compute_aabb() {
            commands.entity(grid.0).insert(aabb);
        }
    }
}

fn gridvec(
    aabb: Box2D<i32, SchematicSpace>,
) -> Vec<Vec3> {
    if aabb.height() > 10000 || aabb.width() > 10000 {
        return vec![];
    }
    let spacing;
    if aabb.height() < 100 && aabb.width() < 100 {
        spacing = 2;
    } else {
        spacing = 16;
    }
    let height = aabb.height() / spacing;
    let width = aabb.width() / spacing;
    let minpoint = ((aabb.min / spacing) + Point2D::splat(1).to_vector()) * spacing;
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