use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities, world::unsafe_world_cell::UnsafeWorldCell, system::CommandQueue},
    prelude::*,
    tasks::IoTaskPool,
    utils::HashSet, scene::serde::SceneDeserializer,
};
use bevy_inspector_egui::egui::color_picker::color_picker_color32;
use serde::de::DeserializeSeed;
use space_shared::{EditorPrefabPath, PrefabMarker, PrefabMemoryCache};
use std::{any::TypeId, fs::File, io::Write};

use crate::prelude::{EditorRegistry, EditorRegistryExt, SceneAutoRoot, SceneAutoChild};

#[derive(Reflect, Default, Component, Clone)]
#[reflect(Component, MapEntities)]
/// Component that holds children entity/prefab information
/// that should be serialized
pub struct ChildrenPrefab(pub Vec<Entity>);

impl ChildrenPrefab {
    pub fn from_children(children: &Children) -> Self {
        Self(children.to_vec())
    }
}

impl MapEntities for ChildrenPrefab {
    fn map_entities(&mut self, entity_mapper: &mut bevy::ecs::entity::EntityMapper) {
        self.0 = self
            .0
            .iter()
            .map(|e| entity_mapper.get_or_reserve(*e))
            .collect();
    }
}

pub struct SavePrefabPlugin;

impl Plugin for SavePrefabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<ChildrenPrefab>();


        app.init_resource::<SaveConfig>().add_state::<SaveState>();

        app.add_systems(
            OnEnter(SaveState::Save),
            (
                prepare_children,
                apply_deferred,
                serialize_scene,
                delete_prepared_children,
            )
                .chain(),
        );
    }
}

/// This struct determine path to save prefab
#[derive(Resource, Clone, Default)]
pub struct SaveConfig {
    pub path: Option<EditorPrefabPath>,
}

/// State system using to enable slow logic of saving
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SaveState {
    Save,
    #[default]
    Idle,
}


fn prepare_children(mut commands: Commands, query: Query<(Entity, &Children), (With<PrefabMarker>, Without<SceneAutoRoot>)>) {
    for (entity, children) in query.iter() {
        commands
            .entity(entity)
            .insert(ChildrenPrefab::from_children(children));
    }
}

fn delete_prepared_children(mut commands: Commands, query: Query<Entity, With<ChildrenPrefab>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<ChildrenPrefab>();
    }
}

/// Convert world scene to prefab
pub fn serialize_scene(world: &mut World) {
    let config = world.resource::<SaveConfig>().clone();

    let mut prefab_query = world.query_filtered::<Entity, (With<PrefabMarker>, Without<SceneAutoChild>)>();
    let entities = prefab_query.iter(world).collect::<Vec<_>>();

    let registry = world.resource::<EditorRegistry>().clone();
    let allow_types: Vec<TypeId> = registry
        .registry
        .read()
        .iter()
        .map(|a| a.type_id())
        .collect();
    let mut builder = DynamicSceneBuilder::from_world(world);
    builder = builder
        .allow_all()
        .with_filter(SceneFilter::Allowlist(HashSet::from_iter(
            allow_types.iter().cloned(),
        )))
        .extract_entities(entities.iter().copied());
    let scene = builder.build();

    let res = scene.serialize_ron(world.resource::<AppTypeRegistry>());

    if let Ok(str) = res {
        // Write the scene RON data to file
        let path = config.path;
        if let Some(path) = path {
            match path {
                EditorPrefabPath::File(path) => {
                    IoTaskPool::get()
                        .spawn(async move {
                            File::create(format!("assets/{path}"))
                                .and_then(|mut file| file.write(str.as_bytes()))
                                .expect("Error while writing scene to file");
                            info!("Saved prefab to file {}", path);
                        })
                        .detach();
                }
                EditorPrefabPath::MemoryCahce => {
                    let handle = world.resource_mut::<Assets<DynamicScene>>().add(scene);
                    world.resource_mut::<PrefabMemoryCache>().scene = Some(handle);
                }
            }
        }
    } else if let Err(e) = res {
        error!("failed to serialize prefab: {:?}", e);
    }

    world
        .resource_mut::<NextState<SaveState>>()
        .set(SaveState::Idle);
}
