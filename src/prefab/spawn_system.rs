use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use super::component::*;

pub fn spawn_scene(
    mut commands : Commands,
    prefabs : Query<(Entity, &ScenePrefab, Option<&Children>), Changed<ScenePrefab>>,
    auto_childs : Query<&PrefabAutoChild>,
    asset_server : Res<AssetServer>
) {
    for (e, prefab, children) in prefabs.iter() {

        if let Some(children) = children {
            for e in children {
                if auto_childs.contains(*e) {
                    commands.entity(*e).despawn_recursive();
                }
            }
        }

        let id = commands.spawn(HookedSceneBundle {
              scene : SceneBundle { 
                scene: asset_server.load(format!("{}#{}", &prefab.path, &prefab.scene)), 
                ..default() },
                hook : SceneHook::new(|e, cmd| {
                    cmd.insert(PrefabAutoChild);
                })
             })
             .insert(PrefabAutoChild)
            .id();
        commands.entity(e).add_child(id);
        commands.entity(e).
                insert(VisibilityBundle::default());
        commands.entity(e).insert(GlobalTransform::default());
    }
}