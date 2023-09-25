
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
pub use bevy_inspector_egui::prelude::*;

use crate::prefab::component::MeshPrimitivePrefab;

use super::RigidBodyPrefab;

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;

#[derive(Reflect, Debug, Clone, PartialEq, Component, InspectorOptions)]
#[reflect(Component, Default)]
pub enum ColliderPrefab {
    FromMesh,
    FromPrefabMesh,
    Primitive{pos : Vec3, rot : Vec3, primitive : ColliderPrimitive},
    Compound(ColliderPrefabCompound)
}

#[derive(Reflect, Debug, Clone, PartialEq, Default)]
#[reflect(Default)]
pub struct ColliderPrefabCompound {
    pub parts : Vec<ColliderPart>
}

impl Default for ColliderPrefab {
    fn default() -> Self {
        ColliderPrefab::Primitive { pos: Vec3::default(), rot: Vec3::default(), primitive: ColliderPrimitive::Cuboid(Vec3::ONE) }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Default)]
#[reflect(Default)]
pub struct ColliderPart {
    pub pos : Vec3,
    pub rot : Vec3,
    pub primitive : ColliderPrimitive
}

#[derive(Debug, Reflect, Clone, PartialEq)]
#[reflect(Default)]
pub enum ColliderPrimitive {
    Cuboid(Vector),
    Capsule{height : f32, radius : f32},
    CapsuleEndpoints{a : Vector, b : Vector, radius : f32},
    Cone{height : f32, radius : f32},
    Cylinder{height : f32, radius : f32},
    Halfspace{outward_normal : Vector},
    Triangle{a : Vector, b : Vector, c : Vector},
    Ball(f32),
    Segment{a : Vector, b : Vector},
}


impl Default for ColliderPrimitive {
    fn default() -> Self {
        ColliderPrimitive::Cuboid(Vector::new(1.0, 1.0, 1.0))
    }
}

impl ColliderPrimitive {
    pub fn to_collider(&self) -> Collider {
        match self {
            ColliderPrimitive::Cuboid(bbox) => {
                Collider::cuboid(bbox.x, bbox.y, bbox.z)
            },
            ColliderPrimitive::Capsule { height, radius } => Collider::capsule(*height, *radius),
            ColliderPrimitive::CapsuleEndpoints { a, b, radius } => Collider::capsule_endpoints(*a, *b, *radius),
            ColliderPrimitive::Cone { height, radius } => Collider::cone(*height, *radius),
            ColliderPrimitive::Cylinder { height, radius } => Collider::cylinder(*height, *radius),
            ColliderPrimitive::Halfspace { outward_normal } => Collider::halfspace(*outward_normal),
            ColliderPrimitive::Triangle { a, b, c } => Collider::triangle(*a, *b, *c),
            ColliderPrimitive::Ball(radius) => Collider::ball(*radius),
            ColliderPrimitive::Segment { a, b } => Collider::segment(*a, *b),
        }
    }
}

pub fn update_collider(
    mut commands : Commands,
    query : Query<(Entity, &ColliderPrefab, Option<&RigidBodyPrefab>, Option<&Transform>, Option<&Handle<Mesh>>, Option<&MeshPrimitivePrefab>), Changed<ColliderPrefab>>,
    updated_meshs : Query<(Entity, &ColliderPrefab, &Handle<Mesh>), Changed<Handle<Mesh>>>,
    updated_prefab_meshs : Query<(Entity, &ColliderPrefab, &MeshPrimitivePrefab), Changed<MeshPrimitivePrefab>>,
    meshs : Res<Assets<Mesh>>
) {
    for (e, collider, rigidbody, transform, mesh, prefab_mesh) in query.iter() {
        commands.entity(e).remove::<Collider>();
        let col = get_collider(collider, mesh, &meshs, prefab_mesh);  
        commands.entity(e).insert(col);

        if rigidbody.is_none() {
            commands.entity(e).insert(RigidBodyPrefab::Static);
        }

        if transform.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }

    for (e, collider, mesh) in updated_meshs.iter() {
        if *collider == ColliderPrefab::FromMesh {
            if let Some(mesh) = meshs.get(mesh) {
                if let Some(col) = Collider::convex_decomposition_from_bevy_mesh(mesh) {
                    commands.entity(e).insert(col);
                } else {
                    commands.entity(e).insert(Collider::trimesh_from_bevy_mesh(mesh).unwrap_or_default());
                }
            } else {
                commands.entity(e).insert(Collider::default());
            }
        }
    }

    for (e, collider, mesh) in updated_prefab_meshs.iter() {
        if *collider == ColliderPrefab::FromPrefabMesh {
            commands.entity(e).remove::<Collider>();
            commands.entity(e).insert(get_prefab_mesh_collider(mesh));
        }
    }
}

fn get_collider(collider: &ColliderPrefab, mesh: Option<&Handle<Mesh>>, meshs: &Assets<Mesh>, prefab_mesh: Option<&MeshPrimitivePrefab>) -> Collider {
    match collider {
        ColliderPrefab::FromMesh => {
            if let Some(mesh) = mesh {
                if let Some(mesh) = meshs.get(mesh) {
                    return Collider::trimesh_from_bevy_mesh(mesh).unwrap_or_default();
                } else {
                    return Collider::default();
                } 
            } else {
                return Collider::default();
            }
        },
        ColliderPrefab::FromPrefabMesh => {
            if let Some(mesh) = prefab_mesh {
                let col = get_prefab_mesh_collider(mesh);
                return col;
            } else {
                return Collider::default();
            }
        },
        ColliderPrefab::Primitive { pos, rot, primitive } => {
            return Collider::compound(vec![(*pos, Quat::from_euler(EulerRot::XYZ, rot.x, rot.y, rot.z), primitive.to_collider())]);
        },
        ColliderPrefab::Compound(com) => {
            if com.parts.len() > 0 {
                return Collider::compound(
                    com.parts.iter().map(|p| (p.pos,  Quat::from_euler(EulerRot::XYZ, p.rot.x, p.rot.y, p.rot.z), p.primitive.to_collider())).collect()
                );
            } else {
                return Collider::default();
            }
        }
    }
}

fn get_prefab_mesh_collider(mesh: &MeshPrimitivePrefab) -> Collider {
    const EPS : f32 = 0.00001;
    let col = match mesh {
        MeshPrimitivePrefab::Cube(val) => Collider::cuboid(*val, *val, *val),
        MeshPrimitivePrefab::Box(val) => Collider::cuboid(val.w, val.h, val.d),
        MeshPrimitivePrefab::Sphere(val) => Collider::ball(val.r),
        MeshPrimitivePrefab::Quad(val) => Collider::cuboid(val.size.x, val.size.y, EPS),
        MeshPrimitivePrefab::Capsule(val) => Collider::capsule(1.0, val.r), 
        MeshPrimitivePrefab::Circle(val) =>Collider::trimesh_from_bevy_mesh(&val.to_mesh()).unwrap_or_default(),
        MeshPrimitivePrefab::Cylinder(val) => Collider::cylinder(1.0, val.r),
        MeshPrimitivePrefab::Icosphere(val) => Collider::trimesh_from_bevy_mesh(&val.to_mesh()).unwrap_or_default(),
        MeshPrimitivePrefab::Plane(val) => Collider::cuboid(val.size, EPS, val.size),
        MeshPrimitivePrefab::RegularPoligon(val) => Collider::trimesh_from_bevy_mesh(&val.to_mesh()).unwrap_or_default(),
        MeshPrimitivePrefab::Torus(val) => Collider::trimesh_from_bevy_mesh(&val.to_mesh()).unwrap_or_default(),
    };
    col
}

// pub fn debug_draw_collider(
//     mut gizmo : Gizmos,
//     query : Query<(Entity, &Collider), Changed<ColliderPrefab>>
// ) {

// }