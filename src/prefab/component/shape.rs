
use crate::ext::*;

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
    RegularPoligon(RegularPoligonPrefab),
    Torus(TorusPrefab),
}

impl Default for MeshPrimitivePrefab {
    fn default() -> Self {
        MeshPrimitivePrefab::Box(BoxPrefab { w: 1.0, h: 1.0, d: 1.0 })
    }
}

impl MeshPrimitivePrefab {
    pub fn to_mesh(&self) -> Mesh {
        match self {
            MeshPrimitivePrefab::Cube(s) => Mesh::from(shape::Cube::new(*s)),
            MeshPrimitivePrefab::Box(b) => b.to_mesh(),
            MeshPrimitivePrefab::Sphere(s) => s.to_mesh(),
            MeshPrimitivePrefab::Quad(q) => q.to_mesh(),
            MeshPrimitivePrefab::Capsule(c) => c.to_mesh(),
            MeshPrimitivePrefab::Circle(c) => c.to_mesh(),
            MeshPrimitivePrefab::Cylinder(c) => c.to_mesh(),
            MeshPrimitivePrefab::Icosphere(c) => c.to_mesh(),
            MeshPrimitivePrefab::Plane(c) => c.to_mesh(),
            MeshPrimitivePrefab::RegularPoligon(c) => c.to_mesh(),
            MeshPrimitivePrefab::Torus(c) => c.to_mesh()
        }
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct BoxPrefab {
    pub w : f32,
    pub h : f32,
    pub d : f32
}

impl Default for BoxPrefab {
    fn default() -> Self {
        Self {
            w : 1.0,
            h : 1.0,
            d : 1.0
        }
    }
}

impl BoxPrefab {
    fn to_mesh(&self) -> Mesh {
        Mesh::from(shape::Box::new(self.w, self.h, self.d))
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct SpherePrefab {
    pub r : f32
}

impl Default for SpherePrefab {
    fn default() -> Self {
        Self {
            r : 1.0
        }
    }
}

impl SpherePrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::UVSphere::default();
        data.radius = self.r;
        Mesh::from(data)
    }
}


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
            flip: false
        }
    }
}

impl QuadPrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Quad::default();
        data.size = self.size;
        data.flip = self.flip;

        Mesh::from(data)
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CapsulePrefab {
    pub r : f32,
    pub rings : usize
}

impl Default for CapsulePrefab {
    fn default() -> Self {
        let def = shape::Capsule::default();
        Self {
            r : def.radius,
            rings : def.rings
        }
    }
}

impl CapsulePrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Capsule::default();
        data.radius = self.r;
        data.rings = self.rings;
        Mesh::from(data)
    }
}

#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct CirclePrefab {
    pub r : f32,
    #[inspector(min = 3)]
    pub vertices : usize
}

impl Default for CirclePrefab {
    fn default() -> Self {
        let def = shape::Circle::default();
        Self {
            r : def.radius,
            vertices : def.vertices
        }
    }
}

impl CirclePrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Circle::default();
        data.radius = self.r;
        data.vertices = self.vertices;
        Mesh::from(data)
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct CylinderPrefab {
    pub r : f32,
    pub resolution : u32,
    pub segments : u32
}

impl Default for CylinderPrefab {
    fn default() -> Self {
        let def = shape::Cylinder::default();
        Self {
            r : def.radius,
            resolution : def.resolution,
            segments : def.segments
        }
    }
}

impl CylinderPrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Cylinder::default();
        data.radius = self.r;
        data.resolution = self.resolution;
        data.segments = self.segments;
        Mesh::from(data)
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct IcospherePrefab {
    pub r : f32,
    pub subdivisions : usize,
}

impl Default for IcospherePrefab {
    fn default() -> Self {
        let def = shape::Icosphere::default();
        Self {
            r : def.radius,
            subdivisions : def.subdivisions
        }
    }
}

impl IcospherePrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Icosphere::default();
        data.radius = self.r;
        data.subdivisions = self.subdivisions;
        if let Ok(mesh) = Mesh::try_from(data) {
            mesh
        } else {
            Mesh::try_from(shape::Icosphere::default()).unwrap()
        }
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct PlanePrefab {
    pub size : f32,
    pub subdivisions : u32,
}

impl Default for PlanePrefab {
    fn default() -> Self {
        let def = shape::Plane::default();
        Self {
            size : def.size,
            subdivisions : def.subdivisions
        }
    }
}

impl PlanePrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Plane::default();
        data.size = self.size;
        data.subdivisions = self.subdivisions;
        Mesh::from(data)
    }
}

#[derive(Reflect, Clone, InspectorOptions)]
#[reflect(Default, InspectorOptions)]
pub struct RegularPoligonPrefab {
    pub radius : f32,
    #[inspector(min = 3)]
    pub sides : usize,
}

impl Default for RegularPoligonPrefab {
    fn default() -> Self {
        let def = shape::RegularPolygon::default();
        Self {
            radius : def.radius,
            sides : def.sides
        }
    }
}

impl RegularPoligonPrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::RegularPolygon::default();
        data.radius = self.radius;
        data.sides = self.sides;
        Mesh::from(data)
    }
}

#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct TorusPrefab {
    pub radius : f32,
    pub ring_radius : f32,
    pub subdivisions_sides : usize,
    pub subdivisions_segments : usize,
}

impl Default for TorusPrefab {
    fn default() -> Self {
        let def = shape::Torus::default();
        Self {
            radius : def.radius,
            ring_radius : def.ring_radius,
            subdivisions_sides : def.subdivisions_sides,
            subdivisions_segments : def.subdivisions_segments
        }
    }
}

impl TorusPrefab {
    fn to_mesh(&self) -> Mesh {
        let mut data = shape::Torus::default();
        data.radius = self.radius;
        data.ring_radius = self.ring_radius;
        data.subdivisions_sides = self.subdivisions_sides;
        data.subdivisions_segments = self.subdivisions_segments;
        Mesh::from(data)
    }
}