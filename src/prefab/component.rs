use bevy::prelude::*;

#[derive(Component, Reflect)]
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