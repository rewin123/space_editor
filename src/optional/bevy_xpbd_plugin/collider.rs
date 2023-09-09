
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::prefab::component::MeshPrimitivePrefab;

use super::RigidBodyPrefab;

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;

#[derive(Debug, Component, Reflect, Clone, PartialEq)]
#[reflect(Component)]
pub enum ColliderPrefab {
    Cuboid(Vector),
    Capsule{height : f32, radius : f32},
    CapsuleEndpoints{a : Vector, b : Vector, radius : f32},
    Cone{height : f32, radius : f32},
    Cylinder{height : f32, radius : f32},
    Halfspace{outward_normal : Vector},
    Triangle{a : Vector, b : Vector, c : Vector},
    Ball(f32),
    Segment{a : Vector, b : Vector},
    FromMesh,
    FromPrefabMesh
}

impl Default for ColliderPrefab {
    fn default() -> Self {
        ColliderPrefab::Cuboid(Vector::new(1.0, 1.0, 1.0))
    }
}

impl ColliderPrefab {
    pub fn to_collider(&self) -> Collider {
        match self {
            ColliderPrefab::Cuboid(bbox) => {
                Collider::cuboid(bbox.x, bbox.y, bbox.z)
            },
            ColliderPrefab::Capsule { height, radius } => Collider::capsule(*height, *radius),
            ColliderPrefab::CapsuleEndpoints { a, b, radius } => Collider::capsule_endpoints(*a, *b, *radius),
            ColliderPrefab::Cone { height, radius } => (Collider::cone(*height, *radius)),
            ColliderPrefab::Cylinder { height, radius } => (Collider::cylinder(*height, *radius)),
            ColliderPrefab::Halfspace { outward_normal } => (Collider::halfspace(*outward_normal)),
            ColliderPrefab::Triangle { a, b, c } => (Collider::triangle(*a, *b, *c)),
            ColliderPrefab::Ball(radius) => Collider::ball(*radius),
            ColliderPrefab::Segment { a, b } => Collider::segment(*a, *b),
            ColliderPrefab::FromMesh => Collider::default(),
            ColliderPrefab::FromPrefabMesh => Collider::default(),
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
    if *collider == ColliderPrefab::FromMesh {
        if let Some(mesh) = mesh {
            if let Some(mesh) = meshs.get(mesh) {
                return Collider::trimesh_from_bevy_mesh(mesh).unwrap_or_default();
            } else {
                return Collider::default();
            } 
        } else {
            return Collider::default();
        }
    } else if *collider == ColliderPrefab::FromPrefabMesh {
        if let Some(mesh) = prefab_mesh {
            let col = get_prefab_mesh_collider(mesh);
            return col;
        } else {
            return Collider::default();
        }
    } else {
        return collider.to_collider();
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