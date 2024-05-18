use bevy::{
    math::bounding::Aabb3d,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use crate::schematic::{SchematicElement, SchematicRes};

// states

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum WiringToolState {
    #[default]
    Ready, // ready to place first anchor
    Drawing(ActiveWireSeg), // placing second anchor point
}

use super::{
    CloneToEnt, ElementType, Preview, SchematicToolState, Selected, Tentative
};

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WireSeg {
    p0: IVec2,
    p1: IVec2,
}

impl WireSeg {
    fn new(pt: IVec2) -> WireSeg {
        WireSeg { p0: pt, p1: pt }
    }
}


#[derive(Bundle)]
pub struct WireSegBundle {
    wireseg: WireSeg,
    meshbundle: MaterialMeshBundle<WireMaterial>,
    schematic_element: SchematicElement,
}

impl WireSegBundle {
    fn clone(
        ws: WireSeg,
        meshes: &mut ResMut<Assets<Mesh>>,
        wire_materials: &mut ResMut<Assets<WireMaterial>>,
    ) -> (WireSegBundle, Handle<Mesh>) {
        let pts = vec![ws.p0.as_vec2().extend(0.0), ws.p1.as_vec2().extend(0.0)];
        dbg!(&pts);
        let mesh = Mesh::new(
            PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pts.clone());
        let meshid = meshes.add(mesh);
        let mat = wire_materials.add(WireMaterial {
            color: Color::rgb(0.6, 0.8, 1.0),
            selected: 0.0,
            tentative: 0.0,
            preview: 0.0,
        });
        (
            WireSegBundle {
                wireseg: ws,
                meshbundle: MaterialMeshBundle {
                    mesh: meshid.clone(),
                    material: mat,
                    ..default()
                },
                schematic_element: SchematicElement {
                    aabb: Aabb3d::from_point_cloud(
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        &pts,
                    ),
                },
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
        commands: &mut Commands,
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
        commands
            .entity(self.entityid)
            .insert(SchematicElement {
                aabb: Aabb3d::from_point_cloud(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    &[
                        self.wireseg.p0.as_vec2().extend(0.0),
                        pt.as_vec2().extend(0.0),
                    ],
                ),
            });

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

pub struct WireToolPlugin;

impl Plugin for WireToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WireMaterial>::default());
        // app.add_systems(OnEnter(SchematicToolState::Wiring), setup);
        app.add_systems(
            Update,
            (
                main.run_if(in_state(super::SchematicToolState::Wiring)),
                set_material,
                append_components,
            ),
        );
        app.add_systems(OnExit(SchematicToolState::Wiring), tool_cleanup);

        app.init_state::<WiringToolState>();
    }
}

// set material based on tentative and selection
fn set_material(
    mut wq: Query<
        (
            &Handle<WireMaterial>,
            Option<&Tentative>,
            Option<&Selected>,
            Option<&Preview>,
        ),
        With<WireSeg>,
    >,
    mut materials: ResMut<Assets<WireMaterial>>,
) {
    for (matid, has_tentative, has_selected, has_preview) in wq.iter_mut() {
        if let Some(mat) = materials.get_mut(matid) {
            mat.tentative = has_tentative.is_some() as i32 as f32;
            mat.selected = has_selected.is_some() as i32 as f32;
            mat.preview = has_preview.is_some() as i32 as f32;
        }
    }
}

fn tool_setup() {
    // this system is deisgned to be run everytime the tool is activated
}

fn tool_cleanup(mut next_wiretoolstate: ResMut<NextState<WiringToolState>>) {
    // this system is deisgned to be run everytime the tool is deactivated
    next_wiretoolstate.set(WiringToolState::Ready);
}

fn main(
    schematic_res: Res<SchematicRes>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    wiretoolstate: Res<State<WiringToolState>>,
    mut next_wiretoolstate: ResMut<NextState<WiringToolState>>,
    mut next_schematictoolstate: ResMut<NextState<SchematicToolState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wire_materials: ResMut<Assets<WireMaterial>>,
) {
    // run if tool state is wire
    match wiretoolstate.get() {
        WiringToolState::Ready => {
            if buttons.just_released(MouseButton::Left) {
                // add entity, change state
                if let Some(pt) = schematic_res.cursor_position.opt_ssp {
                    let (bundle, meshid) = WireSegBundle::clone(WireSeg::new(pt), &mut meshes, &mut wire_materials);
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
                        let (bundle, meshid) = WireSegBundle::clone(WireSeg::new(pt), &mut meshes, &mut wire_materials);
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
                // cant use event since for some reason mesh lags by 1 event
                // if let Some(new_curpos) = e_new_ssp.read().last() {
                //     if let Some(curpos_ssp) = new_curpos.0 {
                //         next_wiretoolstate.set(WiringToolState::Drawing(
                //             aws.new_endpoint(curpos_ssp, commands, meshes),
                //         ));
                //     }
                // }
                if let Some(pt) = schematic_res.cursor_position.opt_ssp {
                    let new_aws = aws.new_endpoint(pt, &mut commands, meshes);
                    commands.entity(new_aws.entityid).insert(new_aws.wireseg.clone());
                    next_wiretoolstate.set(WiringToolState::Drawing(new_aws));
                }
            }
        }
    }
}

// bevy doesnt have an easy way to clone entities yet...
/// this function is used to fill out componenets that is easier to add in this module, e.g. mesh and material
/// conceived for cloning entities. Cloner creates an entity and adds data compoennt, which are both passed here
/// through an event.
fn append_components(
    mut ce: EventReader<CloneToEnt>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wire_materials: ResMut<Assets<WireMaterial>>,
) {
    for CloneToEnt((e, et)) in ce.read() {
        if let ElementType::WireSeg(ws) = et {
            let (wb, _mesh) = WireSegBundle::clone(ws.clone(), &mut meshes, &mut wire_materials);
            commands.entity(*e).insert(wb);
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WireMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    selected: f32,
    #[uniform(2)]
    tentative: f32,
    #[uniform(3)]
    preview: f32,
}

impl Material for WireMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        // let vertex_layout = layout.get_layout(&[Mesh::ATTRIBUTE_POSITION.at_shader_location(0)])?;
        // descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        "schematic_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "schematic_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
