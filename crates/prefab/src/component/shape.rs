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
    Icosphere(IcospherePrefab),
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
        Self::Quad(QuadPrefab {
            size: Vec2::ONE,
            flip: false,
        })
    }
}

impl MeshPrimitivePrefab {
    /// Convert [`MeshPrimitivePrefab`] to bevy [`Mesh`]
    pub fn to_mesh(&self) -> Mesh {
        match self {
            Self::Cube(s) => Mesh::from(shape::Cube::new(*s)),
            Self::Box(b) => b.to_mesh(),
            Self::Sphere(s) => s.to_mesh(),
            Self::Quad(q) => q.to_mesh(),
            Self::Capsule(c) => c.to_mesh(),
            Self::Circle(c) => c.to_mesh(),
            Self::Cylinder(c) => c.to_mesh(),
            Self::Icosphere(c) => c.to_mesh(),
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
        Mesh::from(shape::Box::new(self.w, self.h, self.d))
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
        let data = shape::UVSphere {
            radius: self.r,
            ..Default::default()
        };
        Mesh::from(data)
    }
}

/// Values to setup quad mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct QuadPrefab {
    /// Full width and height of the rectangle.
    pub size: Vec2,
    /// Horizontally-flip the texture coordinates of the resulting mesh.
    pub flip: bool,
}

impl Default for QuadPrefab {
    fn default() -> Self {
        Self {
            size: Vec2::ONE,
            flip: false,
        }
    }
}

impl QuadPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Quad {
            size: self.size,
            flip: self.flip,
        };
        Mesh::from(data)
    }
}

/// Values to setup capsule mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CapsulePrefab {
    pub r: f32,
    pub rings: usize,
}

impl Default for CapsulePrefab {
    fn default() -> Self {
        let def = shape::Capsule::default();
        Self {
            r: def.radius,
            rings: def.rings,
        }
    }
}

impl CapsulePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Capsule {
            radius: self.r,
            rings: self.rings,
            ..Default::default()
        };
        Mesh::from(data)
    }
}

/// Values to setup circle mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct CirclePrefab {
    pub r: f32,
    #[inspector(min = 3)]
    pub vertices: usize,
}

impl Default for CirclePrefab {
    fn default() -> Self {
        let def = shape::Circle::default();
        Self {
            r: def.radius,
            vertices: def.vertices,
        }
    }
}

impl CirclePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Circle {
            radius: self.r,
            vertices: self.vertices,
        };
        Mesh::from(data)
    }
}

/// Values to setup cylinder mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CylinderPrefab {
    pub r: f32,
    pub resolution: u32,
    pub segments: u32,
}

impl Default for CylinderPrefab {
    fn default() -> Self {
        let def = shape::Cylinder::default();
        Self {
            r: def.radius,
            resolution: def.resolution,
            segments: def.segments,
        }
    }
}

impl CylinderPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Cylinder {
            radius: self.r,
            resolution: self.resolution,
            segments: self.segments,
            ..Default::default()
        };
        Mesh::from(data)
    }
}

/// Values to setup icosphere mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct IcospherePrefab {
    pub r: f32,
    pub subdivisions: usize,
}

impl Default for IcospherePrefab {
    fn default() -> Self {
        let def = shape::Icosphere::default();
        Self {
            r: def.radius,
            subdivisions: def.subdivisions,
        }
    }
}

impl IcospherePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Icosphere {
            radius: self.r,
            subdivisions: self.subdivisions,
        };
        Mesh::try_from(data).map_or_else(
            |_| Mesh::try_from(shape::Icosphere::default()).unwrap(),
            |mesh| mesh,
        )
    }
}

/// Values to setup plane mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct PlanePrefab {
    pub size: f32,
    pub subdivisions: u32,
}

impl Default for PlanePrefab {
    fn default() -> Self {
        let def = shape::Plane::default();
        Self {
            size: def.size,
            subdivisions: def.subdivisions,
        }
    }
}

impl PlanePrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Plane {
            size: self.size,
            subdivisions: self.subdivisions,
        };
        Mesh::from(data)
    }
}

/// Values to setup regular polygon mesh
#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct RegularPolygonPrefab {
    pub radius: f32,
    #[inspector(min = 3)]
    pub sides: usize,
}

impl Default for RegularPolygonPrefab {
    fn default() -> Self {
        let def = shape::RegularPolygon::default();
        Self {
            radius: def.radius,
            sides: def.sides,
        }
    }
}

impl RegularPolygonPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::RegularPolygon {
            radius: self.radius,
            sides: self.sides,
        };
        Mesh::from(data)
    }
}

/// Values to setup torus mesh
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct TorusPrefab {
    pub radius: f32,
    pub ring_radius: f32,
    pub subdivisions_sides: usize,
    pub subdivisions_segments: usize,
}

impl Default for TorusPrefab {
    fn default() -> Self {
        let def = shape::Torus::default();
        Self {
            radius: def.radius,
            ring_radius: def.ring_radius,
            subdivisions_sides: def.subdivisions_sides,
            subdivisions_segments: def.subdivisions_segments,
        }
    }
}

impl TorusPrefab {
    pub fn to_mesh(&self) -> Mesh {
        let data = shape::Torus {
            radius: self.radius,
            ring_radius: self.ring_radius,
            subdivisions_sides: self.subdivisions_sides,
            subdivisions_segments: self.subdivisions_segments,
        };
        Mesh::from(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::shape;

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
            format!("{:?}", Mesh::from(shape::Box::new(1.0, 2.0, 3.0)))
        );
    }

    #[test]
    fn test_cube_to_mesh() {
        let cube_prefab = MeshPrimitivePrefab::Cube(2.0);
        let mesh = cube_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(shape::Cube::new(2.0)))
        );
    }

    #[test]
    fn test_sphere_to_mesh() {
        let sphere_prefab = MeshPrimitivePrefab::Sphere(SpherePrefab { r: 1.0 });
        let mesh = sphere_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(shape::UVSphere::default()))
        );
    }

    #[test]
    fn test_icosphere_to_mesh() {
        let sphere_prefab = MeshPrimitivePrefab::Icosphere(IcospherePrefab {
            r: 1.0,
            subdivisions: 5,
        });
        let mesh = sphere_prefab.to_mesh();
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::try_from(shape::Icosphere::default()).unwrap())
        );
    }

    #[test]
    fn test_default_to_mesh() {
        let default_prefab = MeshPrimitivePrefab::default();
        let mesh = default_prefab.to_mesh();
        // You might want to adjust the expected default behavior
        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(shape::Cube::new(1.0)))
        );
    }

    #[test]
    fn test_quad_to_mesh() {
        let default_prefab = MeshPrimitivePrefab::Quad(QuadPrefab {
            size: Vec2::new(1., 1.),
            flip: false,
        });
        let mesh = default_prefab.to_mesh();

        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(shape::Quad::new(Vec2::new(1., 1.))))
        );
    }

    #[test]
    fn test_capsule_to_mesh() {
        let default_prefab = MeshPrimitivePrefab::Capsule(CapsulePrefab::default());
        let mesh = default_prefab.to_mesh();

        assert_eq!(
            format!("{mesh:?}"),
            format!("{:?}", Mesh::from(shape::Capsule::default()))
        );
    }
}
