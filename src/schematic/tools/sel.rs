// tool for making selections
pub struct Sel;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Resource, Default)]
struct SelRes {
    sel_mat_id: Option<Handle<SelMaterial>>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum SelToolState {
    #[default]
    Ready, // ready to place first anchor
    Active(ActiveSelBox), // placing second anchor point
}

use crate::{
    schematic::{NewCurpos, SchematicRes},
    types::{SSBox, SSPoint},
};

use super::SchematicToolState;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct SelBox {
    sel: SSBox,
}

impl SelBox {
    fn new(pt: SSPoint) -> SelBox {
        SelBox {
            sel: SSBox::from_points([pt]),
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SelMaterial {
    #[uniform(0)]
    color_pos: Color,
}

impl Material for SelMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        "sel_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "sel_shader.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Bundle)]
struct SelectionBundle {
    selbox: SelBox,
    meshbundle: MaterialMeshBundle<SelMaterial>,
}

impl SelectionBundle {
    fn new(
        pt: SSPoint,
        mut meshes: ResMut<Assets<Mesh>>,
        sel_mat_id: Handle<SelMaterial>,
    ) -> (SelectionBundle, Handle<Mesh>) {
        let ptf = Vec3::from_array([pt.x as f32, pt.y as f32, 0.0]);
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![ptf, ptf, ptf]);
        let meshid = meshes.add(mesh);
        (
            SelectionBundle {
                selbox: SelBox::new(pt),
                meshbundle: MaterialMeshBundle {
                    mesh: meshid.clone(),
                    material: sel_mat_id,
                    ..default()
                },
            },
            meshid,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActiveSelBox {
    entityid: Entity,
    meshid: Handle<Mesh>,
    selbox: SelBox,
}

impl ActiveSelBox {
    fn new_endpoint(
        &self,
        pt: SSPoint,
        commands: &mut Commands,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> ActiveSelBox {
        let asb = ActiveSelBox {
            entityid: self.entityid,
            meshid: self.meshid.clone(),
            selbox: SelBox {
                sel: SSBox::new(self.selbox.sel.min, pt),
            },
        };
        let mesh = meshes.get_mut(self.meshid.clone()).unwrap();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                Vec3::from_array([
                    self.selbox.sel.min.x as f32,
                    self.selbox.sel.min.y as f32,
                    0.0,
                ]),
                Vec3::from_array([
                    self.selbox.sel.min.x as f32,
                    self.selbox.sel.max.y as f32,
                    0.0,
                ]),
                Vec3::from_array([
                    self.selbox.sel.max.x as f32,
                    self.selbox.sel.min.y as f32,
                    0.0,
                ]),
                Vec3::from_array([
                    self.selbox.sel.max.x as f32,
                    self.selbox.sel.max.y as f32,
                    0.0,
                ]),
                // backtrack so triangles face both directions (negative selections)
                Vec3::from_array([
                    self.selbox.sel.max.x as f32,
                    self.selbox.sel.min.y as f32,
                    0.0,
                ]),
                Vec3::from_array([
                    self.selbox.sel.min.x as f32,
                    self.selbox.sel.max.y as f32,
                    0.0,
                ]),
                Vec3::from_array([
                    self.selbox.sel.min.x as f32,
                    self.selbox.sel.min.y as f32,
                    0.0,
                ]),
            ],
        );

        commands.entity(self.entityid).remove::<Aabb>();
        asb
    }
}

impl Plugin for Sel {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SelMaterial>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Idle)));

        app.init_state::<SelToolState>();
        app.init_resource::<SelRes>();
    }
}

fn setup(
    mut next_selstate: ResMut<NextState<SelToolState>>,
    mut materials: ResMut<Assets<SelMaterial>>,
    mut selres: ResMut<SelRes>,
) {
    // on entering wiring tool
    next_selstate.set(SelToolState::Ready);
    let sel_mat_id = materials.add(SelMaterial {
        color_pos: Color::rgba(1.0, 1.0, 0.0, 0.1),
        // color_neg: Color::AQUAMARINE,
    });
    selres.sel_mat_id = Some(sel_mat_id);
}

fn main(
    schematic_res: Res<SchematicRes>,
    buttons: Res<ButtonInput<MouseButton>>,
    seltoolstate: Res<State<SelToolState>>,
    mut next_seltoolstate: ResMut<NextState<SelToolState>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    selres: Res<SelRes>,
    mut e_new_ssp: EventReader<NewCurpos>,
) {
    // run if tool state is idle
    match seltoolstate.get() {
        SelToolState::Ready => {
            if buttons.just_pressed(MouseButton::Left) {
                // add entity, change state
                if let Some(pt) = schematic_res.cursor_position.opt_ssp {
                    let (bundle, meshid) =
                        SelectionBundle::new(pt, meshes, selres.sel_mat_id.clone().unwrap());
                    let selbox = bundle.selbox.clone();
                    let asel = commands.spawn(bundle).id();
                    next_seltoolstate.set(SelToolState::Active(ActiveSelBox {
                        entityid: asel,
                        meshid,
                        selbox,
                    }));
                }
            }
        }
        SelToolState::Active(asb) => {
            if let Some(new_curpos) = e_new_ssp.read().last() {
                if let Some(curpos_ssp) = new_curpos.0 {
                    next_seltoolstate.set(SelToolState::Active(
                        // also update selected entities
                        asb.new_endpoint(curpos_ssp, &mut commands, meshes),
                    ));
                }
            }
            if buttons.just_released(MouseButton::Left) {
                // remove entity, change state
                commands.entity(asb.entityid).despawn();
                next_seltoolstate.set(SelToolState::Ready);
            }
        }
    }
}
