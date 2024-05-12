use bevy::{
    math::I16Vec2,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

// camera
#[derive(Component)]
struct CameraMarker;

fn setup_camera(
    mut commands: Commands,
) {
    // add camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0., 0., 1.0),
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 0.05,
                ..Default::default()
            }),
            ..default()
        },
        CameraMarker,
        InheritedVisibility::VISIBLE,
    ));
}

// cursor
#[derive(Component)]
struct CursorMarker;

#[derive(Event, Copy, Clone)]
struct NewCurpos(Option<I16Vec2>);

#[derive(Resource, Default)]
struct CurRes {
    curpos: Option<I16Vec2>,
}

fn update_cursor(
    mut curres: ResMut<CurRes>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut e_curpos: EventWriter<NewCurpos>,
) {
    let mut new_curpos = NewCurpos(None);
    if let Ok((camera, cam_transform)) = q_camera.get_single() {
        if let Ok(window) = q_window.get_single() {
            if let Some(coords) = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(cam_transform, cursor))
            {
                new_curpos = NewCurpos(Some(I16Vec2::new(coords.x as i16, coords.y as i16)));
            }
        }
    }

    if curres.curpos != new_curpos.0 {
        e_curpos.send(new_curpos);
        curres.curpos = new_curpos.0;
    }
}

fn draw_cursor(
    mut e_new: EventReader<NewCurpos>,
    mut q_cursor: Query<(&mut Transform, &mut Visibility), With<CursorMarker>>,
) {
    if let Some(NewCurpos(last_e)) = e_new.read().last() {
        if let Some(curpos_ssp) = last_e {
            *q_cursor.single_mut().1 = Visibility::Visible;
            q_cursor.single_mut().0.translation =
                Vec3::new(curpos_ssp.x.into(), curpos_ssp.y.into(), 0.0);
        } else {
            *q_cursor.single_mut().1 = Visibility::Hidden;
        }
    }
}

fn setup_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wire_materials: ResMut<Assets<WireMaterial>>,
    mut color_materials: ResMut<Assets<StandardMaterial>>,
    mut wireres: ResMut<WireRes>,
) {
    // add cursor marker
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes
                .add(bevy::math::primitives::Rectangle::new(0.5, 0.5))
                .into(),
            material: color_materials.add(StandardMaterial{
                base_color: Color::RED,
                emissive: Color::RED,
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        CursorMarker,
    ));
}

// wire
#[derive(Resource, Default)]
struct WireRes {
    wire_mat_id: Option<Handle<WireMaterial>>,
    aws: Option<ActiveWireSeg>,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct WireSeg {
    p0: I16Vec2,
    p1: I16Vec2,
}

impl WireSeg {
    fn new(pt: I16Vec2) -> WireSeg {
        WireSeg { p0: pt, p1: pt }
    }
}

fn update_wire(
    mut wireres: ResMut<WireRes>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut e_new: EventReader<NewCurpos>,
) {
    if let Some(new_curpos) = e_new.read().last() {
        if let Some(curpos) = new_curpos.0 {
            let aws = wireres
                .aws
                .clone()
                .unwrap()
                .new_endpoint(curpos, commands, meshes);
            wireres.aws = Some(aws);
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WireMaterial {
    #[uniform(0)]
    pub color: Color,
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
}

impl WireSegBundle {
    fn new(
        pt: I16Vec2,
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
        pt: I16Vec2,
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
        let aabb = mesh.compute_aabb().unwrap();
        let mut ent = commands.entity(self.entityid);
        ent.insert((self.wireseg.clone(), aabb));

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

fn setup_wire(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wire_materials: ResMut<Assets<WireMaterial>>,
    mut color_materials: ResMut<Assets<StandardMaterial>>,
    mut wireres: ResMut<WireRes>,
) {
    // add a wire
    let wire_mat_id = wire_materials.add(WireMaterial {
        color: Color::WHITE,
    });
    wireres.wire_mat_id = Some(wire_mat_id);

    let (bundle, meshid) = WireSegBundle::new(
        I16Vec2 { x: 0, y: 0 },
        meshes,
        wireres.wire_mat_id.clone().unwrap(),
    );
    let wireseg = bundle.wireseg.clone();
    let aws = commands.spawn(bundle).id();
    wireres.aws = Some(ActiveWireSeg {
        entityid: aws,
        meshid,
        wireseg,
    });
}

// triangle
#[derive(Resource, Default)]
struct TriangleRes {
    wire_mat_id: Option<Handle<StandardMaterial>>,
    aws: Option<ActiveTriangle>,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
struct Triangle {
    pt0: I16Vec2,
    pt1: I16Vec2,
    pt2: I16Vec2,
}

impl Triangle {
    fn new(pt0: I16Vec2, pt1: I16Vec2, pt2: I16Vec2) -> Triangle {
        Triangle { pt0, pt1, pt2 }
    }
}

fn update_triangle(
    mut trangleres: ResMut<TriangleRes>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut e_new: EventReader<NewCurpos>,
) {
    if let Some(new_curpos) = e_new.read().last() {
        if let Some(curpos) = new_curpos.0 {
            let aws = trangleres
                .aws
                .clone()
                .unwrap()
                .new_endpoint(curpos, commands, meshes);
            trangleres.aws = Some(aws);
        }
    }
}

#[derive(Bundle)]
struct TriangleBundle {
    triangle: Triangle,
    meshbundle: MaterialMeshBundle<StandardMaterial>,
}

impl TriangleBundle {
    fn new(
        pts: [I16Vec2; 3],
        mut meshes: ResMut<Assets<Mesh>>,
        mat_id: Handle<StandardMaterial>,
    ) -> (TriangleBundle, Handle<Mesh>) {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![
            Vec3::from_array([pts[0].x as f32, pts[0].y as f32, 0.0]),
            Vec3::from_array([pts[1].x as f32, pts[1].y as f32, 0.0]),
            Vec3::from_array([pts[2].x as f32, pts[2].y as f32, 0.0]),
        ],);
        let meshid = meshes.add(mesh);
        (
            TriangleBundle {
                triangle: Triangle::new(pts[0], pts[1], pts[2]),
                meshbundle: MaterialMeshBundle {
                    mesh: meshid.clone(),
                    material: mat_id,
                    ..default()
                },
            },
            meshid,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActiveTriangle {
    entityid: Entity,
    meshid: Handle<Mesh>,
    triangle: Triangle,
}

impl ActiveTriangle {
    fn new_endpoint(
        &self,
        pt: I16Vec2,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> ActiveTriangle {
        let mesh = meshes.get_mut(self.meshid.clone()).unwrap();
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                Vec3::from_array([self.triangle.pt0.x as f32, self.triangle.pt0.y as f32, 0.0]),
                Vec3::from_array([self.triangle.pt1.x as f32, self.triangle.pt1.y as f32, 0.0]),
                Vec3::from_array([pt.x as f32, pt.y as f32, 0.0]),
            ],
        );
        let aabb = mesh.compute_aabb().unwrap();
        let mut ent = commands.entity(self.entityid);
        ent.insert((self.triangle.clone(), aabb));

        ActiveTriangle {
            entityid: self.entityid,
            meshid: self.meshid.clone(),
            triangle: self.triangle.clone(),
        }
    }
}

fn setup_triangle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<StandardMaterial>>,
    mut trires: ResMut<TriangleRes>,
) {
    // add a wire
    let mat_id = color_materials.add(StandardMaterial {
        base_color: Color::GREEN,
        emissive: Color::GREEN,
        ..Default::default()
    });
    trires.wire_mat_id = Some(mat_id.clone());

    let (bundle, meshid) = TriangleBundle::new(
        [
            I16Vec2{x:0, y:0},
            I16Vec2{x:1, y:1},
            I16Vec2{x:0, y:0},
        ],
        meshes,
        mat_id,
    );
    let triangle = bundle.triangle.clone();
    let aws = commands.spawn(bundle).id();
    trires.aws = Some(ActiveTriangle {
        entityid: aws,
        meshid,
        triangle,
    });
}

// main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<WireMaterial>::default())
        .add_event::<NewCurpos>()
        .add_systems(Update, (update_wire, update_cursor, draw_cursor, update_triangle))
        .add_systems(Startup, (setup_camera, setup_wire, setup_cursor, setup_triangle))
        .init_resource::<CurRes>()
        .init_resource::<WireRes>()
        .init_resource::<TriangleRes>()
        .run();
}




