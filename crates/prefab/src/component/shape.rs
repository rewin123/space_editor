use bevy::math::primitives as math_shapes;
use bevy::prelude::*;
use space_shared::ext::bevy_inspector_egui::prelude::*;

// TODO
// | Line3d |
// | Segment3d |

/// Component to setup mesh of prefab
#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub enum MeshPrimitive3dPrefab {
    Cube(f32),
    Box(BoxPrefab),
    Sphere(SpherePrefab),
    Quad(QuadPrefab),
    Capsule(CapsulePrefab),
    Circle(CirclePrefab),
    Cylinder(CylinderPrefab),
    Plane(Plane3dPrefab),
    PlaneMultipoint(PlaneMultiPointPrefab),
    RegularPolygon(RegularPolygonPrefab),
    Torus(TorusPrefab),
}

#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub enum MeshPrimitive2dPrefab {
    Rectagle(QuadPrefab),
    Circle(CirclePrefab),
    Ellipse(EllipsePrefab),
    Triangle(TrianglePrefab),
    Capsule(Capsule2dPrefab),
    Plane(PlanePrefab),
    RegularPolygon(RegularPolygonPrefab),
}

impl Default for MeshPrimitive3dPrefab {
    fn default() -> Self {
        Self::Box(BoxPrefab {
            w: 1.0,
            h: 1.0,
            d: 1.0,
        })
    }
}

impl Default for MeshPrimitive2dPrefab {
    fn default() -> Self {
        Self::Rectagle(QuadPrefab { size: Vec2::ONE })
    }
}

impl MeshPrimitive3dPrefab {
    /// Convert [`MeshPrimitive3DPrefab`] to bevy [`Mesh`]
    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Cube(s) => Mesh::from(math_shapes::Cuboid::new(*s, *s, *s)),
            Self::Box(b) => b.to_mesh(),
            Self::Sphere(s) => s.to_mesh(),
            Self::Quad(q) => q.to_mesh(),
            Self::Capsule(c) => c.to_mesh(),
            Self::Circle(c) => c.to_mesh(),
            Self::Cylinder(c) => c.to_mesh(),
            Self::Plane(c) => c.to_mesh(),
            Self::RegularPolygon(c) => c.to_mesh(),
            Self::Torus(c) => c.to_mesh(),
            Self::PlaneMultipoint(p) => p.to_mesh(),
        }
    }
}

impl MeshPrimitive2dPrefab {
    /// Convert [`MeshPrimitive2dPrefab`] to bevy [`Mesh`]
    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Rectagle(q) => q.to_mesh(),
            Self::Circle(c) => c.to_mesh(),
            Self::Ellipse(e) => e.to_mesh(),
            Self::Triangle(t) => t.to_mesh(),
            Self::Capsule(c) => c.to_mesh(),
            Self::Plane(c) => c.to_mesh(),
            Self::RegularPolygon(c) => c.to_mesh(),
        }
    }
}

/// Values to setup box mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct BoxPrefab {
    pub w: f32,
    pub h: f32,
    pub d: f32,
}

impl Default for BoxPrefab {
    fn default() -> Self {
        Self {
            w: 1.0,
            h: 1.0,
            d: 1.0,
        }
    }
}

impl BoxPrefab {
    pub fn to_mesh(&self) -> Mesh {
        Mesh::from(math_shapes::Cuboid::new(self.w, self.h, self.d))
    }
}

/// Values to setup sphere mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct SpherePrefab {
    pub r: f32,
}

impl Default for SpherePrefab {
    fn default() -> Self {
        Self { r: 1.0 }
    }
}

impl SpherePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Sphere { radius: self.r };
        Mesh::from(data)
    }
}

/// Values to setup quad mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct QuadPrefab {
    /// Full width and height of the rectangle.
    pub size: Vec2,
}

impl Default for QuadPrefab {
    fn default() -> Self {
        Self {
            size: Vec2::ONE * 10.,
        }
    }
}

impl QuadPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Rectangle::from_size(self.size);
        Mesh::from(data)
    }
}

/// Values to setup capsule mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CapsulePrefab {
    pub r: f32,
    pub half_length: f32,
}

impl Default for CapsulePrefab {
    fn default() -> Self {
        let def = math_shapes::Capsule3d::default();
        Self {
            r: def.radius,
            half_length: def.half_length,
        }
    }
}

impl CapsulePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Capsule3d {
            radius: self.r,
            half_length: self.half_length,
        };
        Mesh::from(data)
    }
}

/// Values to setup circle mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct CirclePrefab {
    pub r: f32,
}

impl Default for CirclePrefab {
    fn default() -> Self {
        let def = math_shapes::Circle::default();
        Self {
            r: def.radius * 10.,
        }
    }
}

impl CirclePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Circle { radius: self.r };
        Mesh::from(data)
    }
}

/// Values to setup ellipse mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct EllipsePrefab {
    pub radius_pair: Vec2,
}

impl Default for EllipsePrefab {
    fn default() -> Self {
        let def = math_shapes::Ellipse::default();
        Self {
            radius_pair: def.half_size * 10.,
        }
    }
}

impl EllipsePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Ellipse {
            half_size: self.radius_pair,
        };
        Mesh::from(data)
    }
}

/// Values to setup Triangle mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct TrianglePrefab {
    pub vertices: [Vec2; 3],
}

impl Default for TrianglePrefab {
    fn default() -> Self {
        let def = math_shapes::Triangle2d::default();
        Self {
            vertices: def.vertices,
        }
    }
}

impl TrianglePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Triangle2d {
            vertices: self.vertices,
        };
        Mesh::from(data)
    }
}

/// Values to setup Capsule2d mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct Capsule2dPrefab {
    pub radius: f32,
    pub half_length: f32,
}

impl Default for Capsule2dPrefab {
    fn default() -> Self {
        let def = math_shapes::Capsule2d::default();
        Self {
            radius: def.radius * 10.,
            half_length: def.half_length * 10.,
        }
    }
}

impl Capsule2dPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Capsule2d {
            radius: self.radius,
            half_length: self.half_length,
        };
        Mesh::from(data)
    }
}

/// Values to setup cylinder mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CylinderPrefab {
    pub r: f32,
    pub half_height: f32,
}

impl Default for CylinderPrefab {
    fn default() -> Self {
        let def = math_shapes::Cylinder::default();
        Self {
            r: def.radius,
            half_height: def.half_height,
        }
    }
}

impl CylinderPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Cylinder {
            radius: self.r,
            half_height: self.half_height,
        };
        Mesh::from(data)
    }
}

/// Values to setup plane mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct PlanePrefab {
    pub size: Vec2,
}

impl Default for PlanePrefab {
    fn default() -> Self {
        let def = math_shapes::Rectangle::default();
        Self {
            size: def.half_size * 2. * 10.,
        }
    }
}

impl PlanePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Rectangle::from_size(self.size * 0.5);
        Mesh::from(data)
    }
}

/// Values to setup Plane3d mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct Plane3dPrefab {
    pub normal: Dir3,
    pub transform: Vec3,
}

impl Default for Plane3dPrefab {
    fn default() -> Self {
        let def = math_shapes::Plane3d::default();
        Self {
            normal: def.normal,
            transform: Vec3::ZERO,
        }
    }
}

impl Plane3dPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Plane3d {
            normal: self.normal,
            half_size: Vec2 { x: 0.5, y: 0.5 },
        };
        Mesh::from(data)
    }

    pub const fn to_plane3d(&self) -> Plane3d {
        math_shapes::Plane3d {
            normal: self.normal,
            half_size: Vec2 { x: 0.5, y: 0.5 },
        }
    }
}

/// Values to setup Plane3d mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct PlaneMultiPointPrefab {
    pub points: [Vec3; 3],
}

impl Default for PlaneMultiPointPrefab {
    fn default() -> Self {
        Self {
            points: [
                Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
                Vec3 {
                    x: 1.,
                    y: 0.,
                    z: 0.,
                },
                Vec3 {
                    x: 0.,
                    y: 0.,
                    z: -1.,
                },
            ],
        }
    }
}

impl PlaneMultiPointPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data =
            math_shapes::Plane3d::from_points(self.points[0], self.points[1], self.points[2]);
        Mesh::from(data.0)
    }

    pub fn to_plane3d(&self) -> Plane3d {
        math_shapes::Plane3d::from_points(self.points[0], self.points[1], self.points[2]).0
    }
}

/// Values to setup regular polygon mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct RegularPolygonPrefab {
    pub circumcircle_radius: f32,
    #[inspector(min = 3)]
    pub sides: usize,
}

impl Default for RegularPolygonPrefab {
    fn default() -> Self {
        let def = math_shapes::RegularPolygon::default();
        Self {
            circumcircle_radius: def.circumcircle.radius * 10.,
            sides: def.sides,
        }
    }
}

impl RegularPolygonPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::RegularPolygon {
            circumcircle: Circle {
                radius: self.circumcircle_radius,
            },
            sides: self.sides,
        };
        Mesh::from(data)
    }
}

/// Values to setup torus mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct TorusPrefab {
    pub minor_radius: f32,
    pub major_radius: f32,
}

impl Default for TorusPrefab {
    fn default() -> Self {
        let def = math_shapes::Torus::default();
        Self {
            minor_radius: def.minor_radius,
            major_radius: def.major_radius,
        }
    }
}

impl TorusPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Torus {
            minor_radius: self.minor_radius,
            major_radius: self.major_radius,
        };
        Mesh::from(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_to_mesh() {
        let box_prefab = MeshPrimitive3dPrefab::Box(BoxPrefab {
            w: 1.0,
            h: 2.0,
            d: 3.0,
        });
        let mesh = box_prefab.to_mesh();

        // Hack to test if they are actually equal
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Cuboid::new(1.0, 2.0, 3.0)))
        );
    }

    #[test]
    fn test_cube_to_mesh() {
        let cube_prefab = MeshPrimitive3dPrefab::Cube(2.0);
        let mesh = cube_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Cuboid::new(2.0, 2.0, 2.0)))
        );
    }

    #[test]
    fn test_sphere_to_mesh() {
        let sphere_prefab = MeshPrimitive3dPrefab::Sphere(SpherePrefab { r: 0.5 });
        let mesh = sphere_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Sphere::default()))
        );
    }

    #[test]
    fn test_default_to_mesh() {
        let default_prefab = MeshPrimitive3dPrefab::default();
        let mesh = default_prefab.to_mesh();
        // You might want to adjust the expected default behavior
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Cuboid::new(1.0, 1.0, 1.0)))
        );
    }

    #[test]
    fn test_quad_to_mesh() {
        let default_prefab = MeshPrimitive3dPrefab::Quad(QuadPrefab {
            size: Vec2::new(1., 1.),
        });
        let mesh = default_prefab.to_mesh();

        assert_eq!(
            format!("{mesh:?}"),
            format!(
                "{:?}",
                Mesh::from(math_shapes::Rectangle::from_size(Vec2::new(1., 1.)))
            )
        );
    }

    #[test]
    fn test_capsule_to_mesh() {
        let default_prefab = MeshPrimitive3dPrefab::Capsule(CapsulePrefab::default());
        let mesh = default_prefab.to_mesh();

        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Capsule3d::default()))
        );
    }

    #[test]
    fn plane_3d_prefab_to_plane3d() {
        let prefab = Plane3dPrefab::default();
        let plane3d = math_shapes::Plane3d {
            normal: Dir3::try_from(Vec3::Y).unwrap(),
            half_size: Vec2 { x: 0.5, y: 0.5 },
        };
        assert_eq!(prefab.to_plane3d(), plane3d);
    }

    #[test]
    fn plane_multipoint_prefab_to_plane3d() {
        let prefab = PlaneMultiPointPrefab::default();
        let plane3d = math_shapes::Plane3d {
            normal: Dir3::try_from(Vec3::Y).unwrap(),
            half_size: Vec2 { x: 0.5, y: 0.5 },
        };
        assert_eq!(prefab.to_plane3d(), plane3d);
    }
}
