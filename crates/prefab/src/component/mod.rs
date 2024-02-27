/// Module contatins structures for determining mesh shapes
pub mod shape;
pub use shape::*;

/// Module contatins structures for determining standard material
pub mod material;
pub use material::*;

/// Module contatins structures for determining sprite
pub mod sprite;
pub use sprite::*;

/// Module contatins structures for determining camera
pub mod camera;
pub use camera::*;

/// Module contatins structures for determining player start
pub mod player_start;
pub use player_start::*;

/// NOT USED. Planned to be used in future for auto structs
pub mod path;

use bevy::{prelude::*, reflect::*, utils::HashMap};

/// External dependencies
pub mod ext {
    pub use space_shared::ext::*;
}

/// Component to define path to gltf asset that will be loaded after prefab spawn
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GltfPrefab {
    pub path: String,
    pub scene: String,
}

impl Default for GltfPrefab {
    fn default() -> Self {
        Self {
            scene: "Scene0".to_string(),
            path: String::new(),
        }
    }
}

/// Marker for entities spawned from gltf scene
#[derive(Component, Reflect, Default)]
pub struct SceneAutoChild;

/// Not used right now. Planned to be easy method for creating prefab structs from usual structs with assets
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct AutoStruct<T: Reflect + Default + Clone> {
    pub data: T,
    pub asset_paths: HashMap<String, String>,
}

impl<T: Reflect + FromReflect + Default + Clone> AutoStruct<T> {
    pub fn new(data: &T, _assets: &AssetServer) -> Self {
        let mut paths = HashMap::new();

        if let ReflectRef::Struct(s) = data.reflect_ref() {
            for idx in 0..s.field_len() {
                let field_name = s.name_at(idx).unwrap();
                let field = s.field_at(idx).unwrap();
                if let Some(handle) = field.downcast_ref::<Handle<Image>>() {
                    if let Some(path) = handle.path() {
                        let path = path.path().to_str().unwrap().to_string();
                        paths.insert(field_name.to_string(), path);
                    }
                }
            }
        }

        Self {
            data: data.clone(),
            asset_paths: paths,
        }
    }

    pub fn get_data(&self, assets: &AssetServer) -> T {
        let mut res = self.data.clone();
        {
            let res_reflect = res.as_reflect_mut();
            if let ReflectMut::Struct(s) = res_reflect.reflect_mut() {
                for (field_name, path) in self.asset_paths.iter() {
                    if let Some(field) = s.field_mut(field_name) {
                        #[allow(clippy::option_if_let_else)]
                        if let Some(handle) = field.downcast_mut::<Handle<Image>>() {
                            *handle = assets.load(path);
                        } else if let Some(handle) = field.downcast_mut::<Handle<Mesh>>() {
                            *handle = assets.load(path);
                        }
                    }
                }
            }
        }
        T::default()
    }
}

/// This component used in prefab to determine links between entities. It is needed to create custom UI in `bevy_inspector_egui`. You must implement the [`MapEntities`](bevy::ecs::entity::MapEntities) trait for your component to make it work. See the `FollowCamera` struct from `examples/platformer.rs`.
#[derive(Reflect, Clone)]
#[reflect(Default)]
pub struct EntityLink {
    pub entity: Entity,
}

impl Default for EntityLink {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
        }
    }
}

/// Component to define path to mesh asset that will be loaded after prefab spawn
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct AssetMesh {
    pub path: String,
}

/// Component to define path to material asset that will be loaded after prefab spawn
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct AssetMaterial {
    pub path: String,
}
