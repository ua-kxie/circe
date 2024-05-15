pub struct Wire;
use bevy::{
    math::bounding::Aabb3d, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, reflect::TypePath, render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    }
};

#[derive(Resource, Default)]
struct WireRes {
    wire_mat_id: Option<Handle<WireMaterial>>,
    tentative_wire_mat_id: Option<Handle<WireMaterial>>,
    selected_wire_mat_id: Option<Handle<WireMaterial>>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum WiringToolState {
    #[default]
    Ready, // ready to place first anchor
    Drawing(ActiveWireSeg), // placing second anchor point
}

use crate::{
    schematic::{NewCurposI, SchematicRes},
};

use super::{sel::{self, SchematicElement}, SchematicToolState};

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct WireSeg {
    p0: IVec2,
    p1: IVec2,
}

impl WireSeg {
    fn new(pt: IVec2) -> WireSeg {
        WireSeg { p0: pt, p1: pt }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct WireMaterial {
    #[uniform(0)]
    pub(crate) color: Color,
}

impl Material for WireMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        let vertex_layout = layout.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        "wire_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "wire_shader.wgsl".into()
    }
}

#[derive(Bundle)]
struct WireSegBundle {
    wireseg: WireSeg,
    meshbundle: MaterialMeshBundle<WireMaterial>,
    schematic_element: SchematicElement,
}

impl WireSegBundle {
    fn new(
        pt: IVec2,
        mut meshes: ResMut<Assets<Mesh>>,
        wire_mat_id: Handle<WireMaterial>,
    ) -> (WireSegBundle, Handle<Mesh>) {
        let ptf = Vec3::from_array([pt.x as f32, pt.y as f32, 0.0]);
        let mesh = Mesh::new(
            PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![ptf, ptf]);
        let meshid = meshes.add(mesh);
        (
            WireSegBundle {
                wireseg: WireSeg::new(pt),
                meshbundle: MaterialMeshBundle {
                    mesh: meshid.clone(),
                    material: wire_mat_id,
                    ..default()
                },
                schematic_element: SchematicElement{
                    aabb: Aabb3d::from_point_cloud(Vec3::ZERO, Quat::IDENTITY, &[pt.as_vec2().extend(0.0)]),
                }
            },
            meshid,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActiveWireSeg {
    entityid: Entity,
    meshid: Handle<Mesh>,
    wireseg: WireSeg,
}

impl ActiveWireSeg {
    fn new_endpoint(
        &self,
        pt: IVec2,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> ActiveWireSeg {
        let mesh = meshes.get_mut(self.meshid.clone()).unwrap();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                Vec3::from_array([self.wireseg.p0.x as f32, self.wireseg.p0.y as f32, 0.0]),
                Vec3::from_array([self.wireseg.p1.x as f32, self.wireseg.p1.y as f32, 0.0]),
            ],
        );

        commands.entity(self.entityid).remove::<Aabb>();
        commands.entity(self.entityid).insert(sel::SchematicElement{
            aabb: Aabb3d::from_point_cloud(
                Vec3::ZERO, 
                Quat::IDENTITY, 
                &[self.wireseg.p0.as_vec2().extend(0.0), pt.as_vec2().extend(0.0)]
            )
        }
        );

        ActiveWireSeg {
            entityid: self.entityid,
            meshid: self.meshid.clone(),
            wireseg: WireSeg {
                p0: self.wireseg.p0,
                p1: pt,
            },
        }
    }
}

impl Plugin for Wire {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WireMaterial>::default());
        app.add_systems(OnEnter(SchematicToolState::Wiring), setup);
        app.add_systems(
            Update,
            (main.run_if(in_state(super::SchematicToolState::Wiring)), set_material),
        );

        app.init_state::<WiringToolState>();
        app.init_resource::<WireRes>();
    }
}

fn setup(
    mut next_wirestate: ResMut<NextState<WiringToolState>>,
    mut materials: ResMut<Assets<WireMaterial>>,
    mut wireres: ResMut<WireRes>,
) {
    // on entering wiring tool
    next_wirestate.set(WiringToolState::Ready);

    wireres.wire_mat_id = Some(materials.add(WireMaterial {
        color: Color::BLUE,
    }));
    wireres.tentative_wire_mat_id = Some(materials.add(WireMaterial {
        color: Color::WHITE,
    }));
    wireres.selected_wire_mat_id = Some(materials.add(WireMaterial {
        color: Color::YELLOW,
    }));
}

// set material based on tentative and selection
fn set_material(
    mut wq_tentatives: Query<&mut Handle<WireMaterial>, With<sel::Tentative>>,
    mut wq_selected: Query<&mut Handle<WireMaterial>, (With<sel::Selected>, Without<sel::Tentative>)>,
    mut wq_defaults: Query<&mut Handle<WireMaterial>, (Without<sel::Selected>, Without<sel::Tentative>)>,
    res_wire_mats: ResMut<WireRes>,
) {
    for mut h in wq_defaults.iter_mut() {
        *h = res_wire_mats.wire_mat_id.clone().unwrap();
    }
    for mut h in wq_tentatives.iter_mut() {
        *h = res_wire_mats.tentative_wire_mat_id.clone().unwrap();
    }
    for mut h in wq_selected.iter_mut() {
        *h = res_wire_mats.selected_wire_mat_id.clone().unwrap();
    }
}

fn main(
    schematic_res: Res<SchematicRes>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wiretoolstate: Res<State<WiringToolState>>,
    mut next_wiretoolstate: ResMut<NextState<WiringToolState>>,
    mut next_schematictoolstate: ResMut<NextState<SchematicToolState>>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    wireres: Res<WireRes>,
    mut e_new_ssp: EventReader<NewCurposI>,
) {
    // run if tool state is wire
    match wiretoolstate.get() {
        WiringToolState::Ready => {
            if buttons.just_released(MouseButton::Left) {
                // add entity, change state
                if let Some(pt) = schematic_res.cursor_position.opt_ssp {
                    let (bundle, meshid) =
                        WireSegBundle::new(pt, meshes, wireres.wire_mat_id.clone().unwrap());
                    let wireseg = bundle.wireseg.clone();
                    let aws = commands.spawn(bundle).id();
                    next_wiretoolstate.set(WiringToolState::Drawing(ActiveWireSeg {
                        entityid: aws,
                        meshid,
                        wireseg,
                    }));
                }
            }
        }
        WiringToolState::Drawing(aws) => {
            if keys.just_released(KeyCode::Escape) {
                commands.entity(aws.entityid).despawn();
                next_schematictoolstate.set(SchematicToolState::Idle);
            } else if buttons.just_released(MouseButton::Left) {
                // left click while a wire seg is being drawn
                // finish adding current entity
                // set up next active wire segment
                // add entity, change state
                if aws.wireseg.p0 != aws.wireseg.p1 {
                    if let Some(pt) = schematic_res.cursor_position.opt_ssp {
                        let (bundle, meshid) =
                            WireSegBundle::new(pt, meshes, wireres.wire_mat_id.clone().unwrap());
                        let wireseg = bundle.wireseg.clone();
                        let aws = commands.spawn(bundle).id();
                        next_wiretoolstate.set(WiringToolState::Drawing(ActiveWireSeg {
                            entityid: aws,
                            meshid,
                            wireseg,
                        }));
                    }
                } else {
                    next_wiretoolstate.set(WiringToolState::Ready);
                }

                // zero length wire segments will be cleaned up during check
            } else {
                // update aws on mouse movement
                if let Some(new_curpos) = e_new_ssp.read().last() {
                    if let Some(curpos_ssp) = new_curpos.0 {
                        next_wiretoolstate.set(WiringToolState::Drawing(
                            aws.new_endpoint(curpos_ssp, commands, meshes),
                        ));
                    }
                }
            }
        }
    }
}
