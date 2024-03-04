use bevy::{
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
};

use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin};

use bevy::sprite::MaterialMesh2dBundle;

pub struct WirePlugin;

impl Plugin for WirePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<CustomMaterial>::default());
        app.add_systems(Startup, wiring_test);
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
        // let vertex_layout = layout.get_layout(&[
        //     Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
        //     ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
        // ])?;
        // descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

fn wiring_test(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials1: ResMut<Assets<CustomMaterial>>,
) {
    let mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 0.0)],
        );
    commands.spawn(MaterialMesh2dBundle {
        // mesh: meshes.add(mesh),
        mesh: meshes.add(mesh).into(),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        // material: materials1.add(WireframeMaterial::default()),
        material: materials1.add(CustomMaterial {
            color: Color::WHITE,
        }),
        ..default()
    });
}
