use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
    tasks::IoTaskPool,
    utils::HashSet,
};
use space_shared::{EditorPrefabPath, PrefabMarker, PrefabMemoryCache};
use std::{any::TypeId, fs, io::Write};

use crate::prelude::{EditorRegistry, EditorRegistryExt, SceneAutoChild};

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
    #[cfg(not(tarpaulin_include))]
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = self
            .0
            .iter()
            .map(|e| entity_mapper.map_entity(*e))
            .collect();
    }
}

struct SaveResourcesPrefabPlugin;

impl Plugin for SaveResourcesPrefabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<ChildrenPrefab>();

        app.init_resource::<SaveConfig>().init_state::<SaveState>();
    }
}

pub struct SavePrefabPlugin;

impl Plugin for SavePrefabPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.add_plugins(SaveResourcesPrefabPlugin {});

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
#[cfg(not(tarpaulin_include))]
#[derive(Resource, Clone, Default)]
pub struct SaveConfig {
    pub path: Option<EditorPrefabPath>,
}

/// State system using to enable slow logic of saving
#[cfg(not(tarpaulin_include))]
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum SaveState {
    Save,
    #[default]
    Idle,
}

fn prepare_children(
    mut commands: Commands,
    query: Query<(Entity, &Children), (With<PrefabMarker>, Without<SceneAutoChild>)>,
) {
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
    let Some(config) = world.get_resource::<SaveConfig>().cloned() else {
        #[cfg(feature = "editor")]
        world.send_event(space_shared::toast::ToastMessage::new(
            "Save config resource not initialized",
            space_shared::toast::ToastKind::Error,
        ));
        error!("Save config resource not initialized");
        return;
    };

    let mut prefab_query =
        world.query_filtered::<Entity, (With<PrefabMarker>, Without<SceneAutoChild>)>();
    let entities = prefab_query.iter(world).collect::<Vec<_>>();

    if entities.is_empty() {
        #[cfg(feature = "editor")]
        world.send_event(space_shared::toast::ToastMessage::new(
            "Saving empty scene",
            space_shared::toast::ToastKind::Warning,
        ));
        warn!("Saving empty scene");
    }

    let Some(registry) = world.get_resource::<EditorRegistry>().cloned() else {
        #[cfg(feature = "editor")]
        world.send_event(space_shared::toast::ToastMessage::new(
            "Editor Registry not initialized",
            space_shared::toast::ToastKind::Error,
        ));
        error!("Editor Registry not initialized");
        return;
    };
    let allow_types: Vec<TypeId> = registry
        .registry
        .read()
        .iter()
        .map(|a| a.type_info().type_id())
        .collect();

    let mut builder = DynamicSceneBuilder::from_world(world);
    builder = builder
        .allow_all()
        .with_filter(SceneFilter::Allowlist(HashSet::from_iter(
            allow_types.iter().cloned(),
        )))
        .extract_entities(entities.iter().copied());
    let scene = builder.build();

    let Some(app_registry) = world.get_resource::<AppTypeRegistry>() else {
        #[cfg(feature = "editor")]
        world.send_event(space_shared::toast::ToastMessage::new(
            "App Registry not initialized",
            space_shared::toast::ToastKind::Error,
        ));
        error!("App Registry not initialized");
        return;
    };

    let res = scene.serialize(&app_registry.read());

    if let Ok(str) = res {
        // Write the scene RON data to file
        let path = config.path;
        if let Some(path) = path {
            match path {
                EditorPrefabPath::File(path) => {
                    IoTaskPool::get()
                        .spawn(async move {
                            fs::OpenOptions::new()
                                .create(true)
                                .truncate(true)
                                .append(false)
                                .write(true)
                                .open(&path)
                                .and_then(|mut file| file.write(str.as_bytes()))
                                .inspect_err(|e| error!("Error while writing scene to file: {e}"))
                                .expect("Error while writing scene to file");
                            info!("Saved prefab to file {}", path);
                        })
                        .detach();
                }
                EditorPrefabPath::MemoryCache => {
                    let handle = world
                        .get_resource_mut::<Assets<DynamicScene>>()
                        .map(|mut assets| assets.add(scene));
                    if let Some(mut cache) = world.get_resource_mut::<PrefabMemoryCache>() {
                        cache.scene = handle;
                    }
                }
            }
        }
    } else if let Err(e) = res {
        // Any ideas on how to test this error case?
        #[cfg(not(tarpaulin_include))]
        let err = format!("failed to serialize prefab: {:?}", e);
        #[cfg(feature = "editor")]
        world.send_event(space_shared::toast::ToastMessage::new(
            &err,
            space_shared::toast::ToastKind::Error,
        ));
        error!(err);
    }

    if let Some(mut state) = world.get_resource_mut::<NextState<SaveState>>() {
        state.set(SaveState::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn flaky_save_to_file() {
        let file = "test.ron";
        let save_config = SaveConfig {
            path: Some(EditorPrefabPath::File(String::from(file))),
        };
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            bevy::scene::ScenePlugin,
            EditorRegistryPlugin {},
            SaveResourcesPrefabPlugin {},
        ))
        .insert_resource(save_config)
        .init_resource::<PrefabMemoryCache>()
        .editor_registry::<Name>()
        .editor_registry::<PrefabMarker>()
        .add_systems(Startup, |mut commands: Commands| {
            let child_id = commands.spawn_empty().id();
            commands.spawn(PrefabMarker).add_child(child_id);

            commands.spawn(PrefabMarker).insert(Name::new("my_name"));
        });

        app.update();

        serialize_scene(&mut app.world_mut());

        // Delay for 0.2 second for IOTaskPool to finish
        std::thread::sleep(std::time::Duration::from_secs_f32(0.2));

        assert!(
            std::fs::metadata(format!("./{}", file)).is_ok(),
            "Flaky Test: File not found"
        );

        let contents = std::fs::read_to_string(file).unwrap();

        assert!(contents.contains("my_name"));
        assert!(contents.contains("space_shared::PrefabMarker"));
    }

    #[test]
    fn save_to_memory() {
        let save_config = SaveConfig {
            path: Some(EditorPrefabPath::MemoryCache),
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_state::<SaveState>();
        app.add_plugins((
            AssetPlugin::default(),
            ImagePlugin::default(),
            bevy::scene::ScenePlugin,
            EditorRegistryPlugin {},
            SaveResourcesPrefabPlugin {},
        ))
        .insert_resource(save_config)
        .init_resource::<PrefabMemoryCache>()
        .editor_registry::<Name>()
        .editor_registry::<PrefabMarker>()
        .add_systems(Startup, |mut commands: Commands| {
            let child_id = commands.spawn_empty().id();
            commands.spawn(PrefabMarker).add_child(child_id);

            commands.spawn(PrefabMarker).insert(Name::new("name"));
        });

        app.update();

        serialize_scene(&mut app.world_mut());
        assert!(app
            .world_mut()
            .resource_mut::<PrefabMemoryCache>()
            .scene
            .is_some());
    }

    #[test]
    fn inserts_prepared_children_component() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let child_id = commands.spawn_empty().id();
            commands.spawn(PrefabMarker).add_child(child_id);

            commands.spawn(PrefabMarker);
        })
        .add_systems(Update, prepare_children);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ChildrenPrefab>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn deletes_prepared_children_component() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let child_id = commands.spawn_empty().id();
            commands
                .spawn(PrefabMarker)
                .insert(ChildrenPrefab(vec![child_id]));
            let child_id = commands.spawn_empty().id();
            commands
                .spawn(PrefabMarker)
                .insert(ChildrenPrefab(vec![child_id]));
            commands.spawn(PrefabMarker);
        })
        .add_systems(Update, delete_prepared_children);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ChildrenPrefab>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 0);
    }

    #[test]
    fn child_prefab_from_children() {
        let mut world = World::new();
        let child = world.spawn_empty().id();
        world.spawn(PrefabMarker).add_child(child);

        let mut query = world.query::<&Children>();
        let children = query.single(&world);
        let prefab = ChildrenPrefab::from_children(children);

        assert_eq!(prefab.0.len(), 1);
    }

    #[test]
    fn attempts_to_serialize_empty_scene() {
        let save_config = SaveConfig {
            path: Some(EditorPrefabPath::MemoryCache),
        };
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            bevy::scene::ScenePlugin,
            EditorRegistryPlugin {},
            SaveResourcesPrefabPlugin {},
        ))
        .add_event::<space_shared::toast::ToastMessage>()
        .insert_resource(save_config)
        .init_resource::<PrefabMemoryCache>();

        app.update();

        serialize_scene(&mut app.world_mut());
        let events = app
            .world_mut()
            .resource::<Events<space_shared::toast::ToastMessage>>();

        let mut iter = events.get_reader();
        let iter = iter.read(events);
        iter.for_each(|e| assert_eq!(e.text, "Saving empty scene"));
    }

    #[test]
    fn prepared_children_ignores_scene_auto_child_component() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let child_id = commands.spawn_empty().id();
            commands
                .spawn((PrefabMarker, SceneAutoChild))
                .add_child(child_id);

            let child_id = commands.spawn_empty().id();
            commands.spawn(PrefabMarker).add_child(child_id);

            commands.spawn(PrefabMarker);
        })
        .add_systems(Update, prepare_children);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ChildrenPrefab>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }
}
