use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

use super::component::*;

/// This system using for spawning gltf scene
pub fn spawn_scene(
    mut commands : Commands,
    prefabs : Query<(Entity, &GltfPrefab, Option<&Children>, Option<&Visibility>, Option<&Transform>), Changed<GltfPrefab>>,
    auto_childs : Query<&SceneAutoChild>,
    asset_server : Res<AssetServer>
) {
    for (e, prefab, children, vis, tr) in prefabs.iter() {

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
                    cmd.insert(SceneAutoChild);
                })
             })
             .insert(SceneAutoChild)
            .id();
        commands.entity(e).add_child(id);

        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }
        if tr.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }
}

/// System to sync Mesh and MeshPrimitivePrefab
pub fn sync_mesh(
    mut commands : Commands,
    query : Query<(Entity, &MeshPrimitivePrefab), Changed<MeshPrimitivePrefab>>,
    mut meshs : ResMut<Assets<Mesh>>
) {
    for (e, pref) in query.iter() {
        let mesh = meshs.add(pref.to_mesh());
        commands.entity(e).insert(mesh);
    }
}

/// System to sync StandartMaterial and MaterialPrefab
pub fn sync_material(
    mut commands : Commands,
    query : Query<(Entity, &MaterialPrefab), Changed<MaterialPrefab>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
    asset_server : Res<AssetServer>
) {
    for (e, pref) in query.iter() {
        let mat = materials.add(pref.to_material(&asset_server));
        commands.entity(e).insert(mat);
    }
}

/// remove mesh handle if prefab struct was removed in editor states
pub fn editor_remove_mesh(
    mut commands : Commands,
    mut query : RemovedComponents<MeshPrimitivePrefab>,
) {
    for e in query.iter() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>();
        }
    }
}

/// Spawn system on enter to EditorState::Game state
pub fn spawn_player_start(
    mut commands : Commands,
    query : Query<(Entity, &PlayerStart)>,
    asset_server : Res<AssetServer>
) {
    for (e, prefab) in query.iter() {
        info!("Spawning player start {:?} {}", e, &prefab.prefab);
        let child = commands.spawn(DynamicSceneBundle {
            scene : asset_server.load(format!("{}", &prefab.prefab)),
            ..default()
        }).id();
        commands.entity(e).add_child(child);
    }
}

// pub fn despawn_player_start(
//     mut commands : Commands,
//     query : Query<Entity, (With<PlayerStart>, With<Handle<Scene>>)>
// ) {
//     for e in query.iter() {

//     }
// }