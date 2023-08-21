use bevy::prelude::*;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GltfPrefab {
    pub path : String,
    pub scene : String
}

impl Default for GltfPrefab {
    fn default() -> Self {
        Self { 
            scene: "Scene0".to_string(),
            path : String::new()
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct ScaneAutoChild;


#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub enum MeshPrimitivePrefab {
    Cube(f32),
    Box(BoxPrefab),
    Sphere(SpherePrefab)
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

#[derive(Component, Reflect, Clone)]
pub struct MaterialPrefab {
    pub color : Color
}

impl Default for MaterialPrefab {
    fn default() -> Self {
        Self { 
            color: Color::GRAY
        }
    }
}