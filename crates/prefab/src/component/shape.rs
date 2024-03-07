use bevy::math::primitives as math_shapes;
use bevy::prelude::*;
use space_shared::ext::bevy_inspector_egui::prelude::*;

/// Component to setup mesh of prefab
#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub enum MeshPrimitivePrefab {
    Cube(f32),
    Box(BoxPrefab),
    Sphere(SpherePrefab),
    Quad(QuadPrefab),
    Capsule(CapsulePrefab),
    Circle(CirclePrefab),
    Cylinder(CylinderPrefab),
    Plane(PlanePrefab),
    RegularPolygon(RegularPolygonPrefab),
    Torus(TorusPrefab),
}

impl Default for MeshPrimitivePrefab {
    fn default() -> Self {
        Self::Box(BoxPrefab {
            w: 1.0,
            h: 1.0,
            d: 1.0,
        })
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub enum MeshPrimitive2dPrefab {
    Quad(QuadPrefab),
    Circle(CirclePrefab),
    Plane(PlanePrefab),
    RegularPolygon(RegularPolygonPrefab),
}

impl Default for MeshPrimitive2dPrefab {
    fn default() -> Self {
        Self::Quad(QuadPrefab { size: Vec2::ONE })
    }
}

impl MeshPrimitivePrefab {
    /// Convert [`MeshPrimitivePrefab`] to bevy [`Mesh`]
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
        }
    }
}

impl MeshPrimitive2dPrefab {
    /// Convert [`MeshPrimitive2dPrefab`] to bevy [`Mesh`]
    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Quad(q) => q.to_mesh(),
            Self::Circle(c) => c.to_mesh(),
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
        Self { size: Vec2::ONE }
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
        Self { r: def.radius }
    }
}

impl CirclePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Circle { radius: self.r };
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
            size: def.half_size * 2.,
        }
    }
}

impl PlanePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = math_shapes::Rectangle::from_size(self.size * 0.5);
        Mesh::from(data)
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
            circumcircle_radius: def.circumcircle.radius,
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
        let box_prefab = MeshPrimitivePrefab::Box(BoxPrefab {
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
        let cube_prefab = MeshPrimitivePrefab::Cube(2.0);
        let mesh = cube_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Cuboid::new(2.0, 2.0, 2.0)))
        );
    }

    #[test]
    fn test_sphere_to_mesh() {
        let sphere_prefab = MeshPrimitivePrefab::Sphere(SpherePrefab { r: 1.0 });
        let mesh = sphere_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Sphere::default()))
        );
    }

    #[test]
    fn test_default_to_mesh() {
        let default_prefab = MeshPrimitivePrefab::default();
        let mesh = default_prefab.to_mesh();
        // You might want to adjust the expected default behavior
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Cuboid::new(1.0, 1.0, 1.0)))
        );
    }

    #[test]
    fn test_quad_to_mesh() {
        let default_prefab = MeshPrimitivePrefab::Quad(QuadPrefab {
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
        let default_prefab = MeshPrimitivePrefab::Capsule(CapsulePrefab::default());
        let mesh = default_prefab.to_mesh();

        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(math_shapes::Capsule3d::default()))
        );
    }
}
