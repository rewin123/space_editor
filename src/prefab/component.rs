use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct ScenePrefab {
    pub path : String,
    pub scene : String
}

impl Default for ScenePrefab {
    fn default() -> Self {
        Self { 
            scene: "Scene0".to_string(),
            path : String::new()
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct PrefabAutoChild;