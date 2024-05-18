// tool for making selections
use crate::{
    schematic::{NewCurposI, SchematicRes},
    types::{NewIVec2, SSBox},
};
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

use super::SchematicToolState;

use bevy::math::bounding::{Aabb3d, AabbCast3d, IntersectsVolume, RayCast3d};

// resources
#[derive(Resource, Default)]
struct SelRes {
    sel_mat_id: Option<Handle<SelMaterial>>,
}

// states
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum SelToolState {
    #[default]
    Ready, // ready to place first anchor
    Active(ActiveSelBox), // placing second anchor point
}

// components
/// marks the selection box used to make area selections
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct SelBox {
    sel: SSBox,
}

impl SelBox {
    fn new(pt: IVec2) -> SelBox {
        SelBox {
            sel: SSBox::new(NewIVec2::from(pt).into(), NewIVec2::from(pt).into()),
        }
    }
}

// marker components
/// componenent marks entity as a schematic element - can be selected, marked as tentative.
/// Entities with this compoenent should be checked for collision against cursor

#[derive(Component)]
pub struct SchematicElement {
    pub aabb: Aabb3d,
}

// /// component is added automatically to SchematicElements without it.
// /// Should be deleted on elements which changes position/shape.
// #[derive(Component)]
// pub struct Collider(Aabb3d);

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Tentative;

// events

/// event fires when elements marked tentatives should be come selected
#[derive(Event)]
struct TentativesToSelected;

/// event fires when elements marked selected should be cleared
#[derive(Event)]
struct ClearSelected;

/// event fires when tentative collider changes (mouse moves, or area selection is dropped)
#[derive(Event)]
pub enum NewTentativeCollider {
    /// no collider, e.g. cursor moved off canvas
    None,
    /// ray collider, selection by point
    Ray(RayCast3d),
    /// volume collider selection by area
    Volume(AabbCast3d),
}

// systems
// setup system to run on startup
fn setup(
    mut next_selstate: ResMut<NextState<SelToolState>>,
    mut materials: ResMut<Assets<SelMaterial>>,
    mut selres: ResMut<SelRes>,
) {
    // on entering wiring tool
    next_selstate.set(SelToolState::Ready);
    let sel_mat_id = materials.add(SelMaterial {
        color_pos: Color::rgba(1.0, 1.0, 0.0, 0.1),
    });
    selres.sel_mat_id = Some(sel_mat_id);
}

// main loop system to handle selections
fn main(
    schematic_res: Res<SchematicRes>,
    buttons: Res<ButtonInput<MouseButton>>,
    seltoolstate: Res<State<SelToolState>>,
    mut next_seltoolstate: ResMut<NextState<SelToolState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    selres: Res<SelRes>,
    mut e_new_ssp: EventReader<NewCurposI>,
    mut e_new_sel_collider: EventWriter<NewTentativeCollider>,
    mut e_tentatives_to_selected: EventWriter<TentativesToSelected>,
) {
    // system runs if tool state is idle/selection mode
    if let Some(new_curpos) = e_new_ssp.read().last() {
        // send new collider event based on new cursor position
        if let Some(pt) = new_curpos.0 {
            match seltoolstate.get() {
                SelToolState::Ready => {
                    let ray = RayCast3d::new(pt.as_vec2().extend(0.0), Direction3d::Z, 1.0);
                    e_new_sel_collider.send(NewTentativeCollider::Ray(ray));
                }
                SelToolState::Active(asb) => {
                    // update selection visual
                    next_seltoolstate.set(SelToolState::Active(
                        // also update selected entities
                        asb.new_endpoint(pt, &mut commands, &mut meshes),
                    ));
                    // send new collider event
                    let pt0: IVec2 = NewIVec2::from(asb.selbox.sel.min).into();
                    let pt1: IVec2 = NewIVec2::from(asb.selbox.sel.max).into();
                    let aabb = Aabb3d::from_point_cloud(
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        &[pt0.as_vec2().extend(0.0), pt1.as_vec2().extend(0.0)],
                    );
                    e_new_sel_collider.send(NewTentativeCollider::Volume(AabbCast3d::new(
                        aabb,
                        Vec3::ZERO,
                        Direction3d::Z,
                        1.0,
                    )));
                }
            }
        } else {
            // todo: make selection visual disappear if applicable
            e_new_sel_collider.send(NewTentativeCollider::None);
        }
    }

    if let Some(pt) = schematic_res.cursor_position.opt_ssp {
        if buttons.just_pressed(MouseButton::Left) {
            // set next tool state to be area selection
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
        if buttons.just_released(MouseButton::Left) {
            e_tentatives_to_selected.send(TentativesToSelected);
        }
    }
    if !buttons.pressed(MouseButton::Left) {
        if let SelToolState::Active(asb) = seltoolstate.get() {
            // remove entity, change state
            commands.entity(asb.entityid).despawn();
            next_seltoolstate.set(SelToolState::Ready);
        }
    }
}

/// this system marks schematic elements with the [`Tentative`] marker if they intersect the selection collider
fn mark_tentatives(
    schematic_elements: Query<(Entity, &SchematicElement)>,
    e_tentatives: Query<Entity, With<Tentative>>,
    mut enew_tentative_collider: EventReader<NewTentativeCollider>,
    mut commands: Commands,
) {
    // mark as tentative if tentative collider changes
    if let Some(collider) = enew_tentative_collider.read().last() {
        // clear tentative markers
        for e in &e_tentatives {
            commands.entity(e).remove::<Tentative>();
        }
        // add tentative markers based on new information
        match collider {
            NewTentativeCollider::None => {}
            NewTentativeCollider::Ray(cast) => {
                for e in schematic_elements.iter() {
                    if cast.intersects(&e.1.aabb) {
                        commands.entity(e.0).insert(Tentative);
                    }
                }
            }
            NewTentativeCollider::Volume(cast) => {
                for e in schematic_elements.iter() {
                    if cast.intersects(&e.1.aabb) {
                        commands.entity(e.0).insert(Tentative);
                    }
                }
            }
        }
    }
}

/// this system simply adds the [`Selected`] marker to all entities already marked with [`Tentative`],
/// conditioned on the event [`TentativesToSelected`]
fn select(
    mut e: EventReader<TentativesToSelected>,
    es: Query<Entity, With<Tentative>>,
    mut commands: Commands,
) {
    if e.read().last().is_some() {
        for element in es.iter() {
            commands.entity(element).insert(Selected);
        }
    }
}

/// this system simply removes the [`Selected`] marker to all entities already marked,
/// conditioned on the event [`ClearSelected`]
fn clr_selected(
    keys: Res<ButtonInput<KeyCode>>,
    es: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for element in es.iter() {
            commands.entity(element).remove::<Selected>();
        }
    }
}

/// this system simply removes all entities marked with Selected,
fn del_selected(
    keys: Res<ButtonInput<KeyCode>>,
    es: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Delete) {
        for element in es.iter() {
            commands.entity(element).despawn();
        }
    }
}

pub struct SelToolPlugin;

impl Plugin for SelToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SelMaterial>::default());
        app.add_systems(Startup, (setup,));
        app.add_systems(
            Update,
            (
                main.run_if(in_state(SchematicToolState::Idle)),
                select,
                clr_selected,
                del_selected,
                mark_tentatives,
            ),
        );

        app.init_state::<SelToolState>();
        app.init_resource::<SelRes>();
        app.add_event::<TentativesToSelected>();
        app.add_event::<ClearSelected>();
        app.add_event::<NewTentativeCollider>();
    }
}

// materials / pipeline

/// this material is used to render the selection box
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

// bundles
#[derive(Bundle)]
struct SelectionBundle {
    selbox: SelBox,
    meshbundle: MaterialMeshBundle<SelMaterial>,
}

impl SelectionBundle {
    fn new(
        pt: IVec2,
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

// helper structs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActiveSelBox {
    entityid: Entity,
    meshid: Handle<Mesh>,
    selbox: SelBox,
}

impl ActiveSelBox {
    fn new_endpoint(
        &self,
        pt: IVec2,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> ActiveSelBox {
        let asb = ActiveSelBox {
            entityid: self.entityid,
            meshid: self.meshid.clone(),
            selbox: SelBox {
                sel: SSBox::new(self.selbox.sel.min, NewIVec2::from(pt).into()),
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
