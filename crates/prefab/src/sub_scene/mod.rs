use std::any::TypeId;

use bevy::{
    ecs::world::unsafe_world_cell::UnsafeWorldCell, prelude::*, reflect::TypeRegistryArc,
    scene::serde::SceneDeserializer, utils::HashSet,
};
use serde::de::DeserializeSeed;
#[cfg(feature = "editor")]
use space_shared::toast::ToastMessage;

use crate::{
    component::*,
    prelude::{ChildrenPrefab, EditorRegistry, EditorRegistryExt, SaveState},
};

pub struct SceneUnpackPlugin;

impl Plugin for SceneUnpackPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, SubScenePersistSet::Prepare);
        app.configure_sets(Update, SubScenePersistSet::Unpack);

        app.add_systems(
            OnEnter(SaveState::Save),
            (
                (prepare_auto_scene, apply_deferred)
                    .chain()
                    .before(crate::prelude::serialize_scene),
                clear_after_save.after(crate::prelude::serialize_scene),
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                decompress_scene,
                apply_deferred,
                apply_compressed_scenes,
                apply_deferred,
            )
                .chain(),
        );

        app.editor_registry::<CollapsedSubScene>();
        app.editor_registry::<ChildPath>();

        app.register_type::<Vec<usize>>();
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SubScenePersistSet {
    Prepare,
    Unpack,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CollapsedSubScene(pub String);

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct ChildPath(pub Vec<usize>);

fn clear_after_save(mut commands: Commands, queue: Query<Entity, With<CollapsedSubScene>>) {
    for entity in queue.iter() {
        commands.entity(entity).remove::<CollapsedSubScene>();
    }
}

pub fn prepare_auto_scene(world: &mut World) {
    unsafe {
        let cell = world.as_unsafe_world_cell();

        // Iter all scene roots
        let mut scene_root_query = cell
            .world_mut()
            .query_filtered::<Entity, With<SceneAutoRoot>>();
        let scene_roots = scene_root_query.iter(cell.world()).collect::<Vec<_>>();

        for root_entity in scene_roots.iter() {
            let Some(registry) = cell
                .world()
                .get_resource::<crate::prelude::EditorRegistry>()
                .cloned()
            else {
                continue;
            };
            let allow_types: Vec<TypeId> = registry
                .registry
                .read()
                .iter()
                .map(|a| a.type_id())
                .collect();

            let mut dyn_scene = DynamicSceneBuilder::from_world(cell.world())
                .allow_all()
                .with_filter(SceneFilter::Allowlist(HashSet::from_iter(
                    allow_types.iter().cloned(),
                )));

            dyn_scene = recursive_extract(&cell, dyn_scene, *root_entity);

            let scene = dyn_scene.build();
            //let Some(app_registry) = cell.world().get_resource::<AppTypeRegistry>() else {
            //    continue;
            //};

            let type_registry_arc: &TypeRegistryArc = &**cell.world().resource::<AppTypeRegistry>();
            let type_registry = type_registry_arc.read();
            let data = scene.serialize(&type_registry);

            if let Ok(data) = data {
                info!("serialized sub scene: {:?}", data);
                cell.world_mut()
                    .entity_mut(*root_entity)
                    .insert(CollapsedSubScene(data));
            } else {
                error!("failed to serialize sub scene: {:?}", data);
            }
        }
    }
}

unsafe fn recursive_extract<'w>(
    cell: &UnsafeWorldCell,
    scene: DynamicSceneBuilder<'w>,
    entity: Entity,
) -> DynamicSceneBuilder<'w> {
    if cell.get_entity(entity).is_some() {
        let mut scene = scene.extract_entity(entity);

        if let Some(children) = cell.world().entity(entity).get::<Children>() {
            for child_entity in children.iter() {
                scene = recursive_extract(cell, scene, *child_entity);
            }
        }
        scene
    } else {
        scene
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct DecompressedScene(pub Scene);

fn decompress_scene(
    mut commands: Commands,
    roots: Query<(Entity, &CollapsedSubScene)>,
    type_registry: Res<AppTypeRegistry>,
    #[cfg(feature = "editor")] mut toast: EventWriter<ToastMessage>,
) {
    for (root_entity, root) in roots.iter() {
        let scene_deserializer = SceneDeserializer {
            type_registry: &type_registry.read(),
        };
        let Ok(mut deserializer) = ron::de::Deserializer::from_str(root.0.as_str()) else {
            #[cfg(feature = "editor")]
            toast.send(ToastMessage::new(
                "Failed create Deserializer for sub scene",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };
        let Ok(dyn_scene) = scene_deserializer.deserialize(&mut deserializer) else {
            #[cfg(feature = "editor")]
            toast.send(ToastMessage::new(
                "Failed to deserialize sub scene",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };

        let Ok(scene) = Scene::from_dynamic_scene(&dyn_scene, &type_registry) else {
            #[cfg(feature = "editor")]
            toast.send(ToastMessage::new(
                "Decompress scene does not exist",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };

        commands
            .entity(root_entity)
            .insert(DecompressedScene(scene))
            .remove::<CollapsedSubScene>();
    }
}

fn apply_compressed_scenes(
    mut commands: Commands,
    mut roots: Query<(Entity, &mut DecompressedScene, &Handle<Scene>, &Children)>,
    child_tree: Query<(Entity, Option<&Children>, Option<&ChildPath>)>,
    editor_registry: Res<EditorRegistry>,
    asset_server: Res<AssetServer>,
) {
    for (root_entity, mut scene, base_scene, children) in roots.iter_mut() {
        if asset_server.load_state(base_scene) != bevy::asset::LoadState::Loaded {
            continue;
        }

        let mut all_sub_entities = HashSet::new();
        let mut stack = vec![root_entity];
        while let Some(entity) = stack.pop() {
            let sub_entities = child_tree.get(entity).unwrap();
            if let Some(children) = sub_entities.1 {
                for child in children.iter() {
                    stack.push(*child);
                    all_sub_entities.insert(*child);
                }
            }
        }

        info!("all sub entities: {:?}", all_sub_entities);
        info!("root entity: {:?}", root_entity);

        let mut scene_query = scene.world.query::<Entity>();

        let scene_entities = scene_query.iter(&scene.world).collect::<Vec<_>>();

        for entity in scene_entities {
            let mut child_path = None;
            if let Some(get_path) = scene.world.entity(entity).get::<ChildPath>() {
                child_path = Some(get_path.clone());
            }

            scene.world.entity_mut(entity).remove::<ChildrenPrefab>();

            if let Some(child_path) = child_path {
                if child_path.0.is_empty() {
                    continue;
                }

                let mut target_entity = root_entity;
                let mut target_children = Some(children);
                for i in child_path.0.iter() {
                    if let Some(children) = target_children {
                        target_entity = *children.get(*i).unwrap();
                        target_children = child_tree.get(target_entity).unwrap().1;
                    } else {
                        error!("failed to find child path");
                        return;
                    }
                }

                if let Some(mut cmds) = commands.get_entity(target_entity) {
                    if all_sub_entities.contains(&target_entity) {
                        all_sub_entities.remove(&target_entity);
                    }
                    for clone_fn in editor_registry.clone_components.iter() {
                        (clone_fn.func)(&mut cmds, &scene.world.entity(entity));
                    }
                }

                scene.world.entity_mut(entity).despawn();
            } else {
                warn!("failed to find child path in sub entity");
            }
        }

        //Destroy not stored entity
        for entity in all_sub_entities.iter() {
            commands.entity(*entity).despawn_recursive();
        }

        commands.entity(root_entity).remove::<DecompressedScene>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_subscene_aftersave() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                space_shared::PrefabMarker,
                CollapsedSubScene(String::from("tes1")),
            ));
            commands.spawn((
                space_shared::PrefabMarker,
                CollapsedSubScene(String::from("test2")),
            ));
        });
        app.add_systems(Update, clear_after_save);
        app.update();
        app.update();

        let mut query = app.world_mut().query::<&CollapsedSubScene>();

        assert_eq!(query.iter(&app.world()).count(), 0);
    }

    #[test]
    #[cfg(feature = "editor")]
    fn decompress_scene_trows_event_when_missing_subscene() {
        let file = "test.ron";

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            bevy::scene::ScenePlugin,
            crate::prelude::EditorRegistryPlugin {},
        ))
        .add_event::<ToastMessage>()
        .init_resource::<space_shared::PrefabMemoryCache>()
        .editor_registry::<Name>()
        .editor_registry::<space_shared::PrefabMarker>();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(CollapsedSubScene(file.to_string()));
        });
        app.add_systems(Update, decompress_scene);
        app.update();

        let events = app.world().resource::<Events<ToastMessage>>();

        let mut iter = events.get_reader();
        let iter = iter.read(events);
        iter.for_each(|e| assert_eq!(e.text, "Failed to deserialize sub scene"));
    }
}
