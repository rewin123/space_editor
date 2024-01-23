use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use space_shared::PrefabMarker;

use super::component::*;

/// System responsible for spawning GLTF objects in the scene
pub fn spawn_scene(
    mut commands: Commands,
    prefabs: Query<
        (
            Entity,
            &GltfPrefab,
            Option<&Children>,
            Option<&Visibility>,
            Option<&Transform>,
            Option<&SceneAutoChild>
        ),
        Changed<GltfPrefab>,
    >,
    auto_childs: Query<&SceneAutoChild>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab, children, vis, tr, auto_child) in prefabs.iter() {
        if let Some(children) = children {
            for e in children {
                if auto_childs.contains(*e) {
                    commands.entity(*e).despawn_recursive();
                }
            }
        }

        let is_auto_child = auto_child.is_some();

        commands.entity(e)
            .insert(asset_server.load::<Scene>(format!("{}#{}", &prefab.path, &prefab.scene)))
            .insert(SceneHook::new(move|_e, cmd| {
                    if _e.contains::<SceneAutoRoot>() {

                    } else {
                        if is_auto_child {
                            cmd.insert(SceneAutoChild);
                        } else {
                            cmd.insert(SceneAutoChild).insert(PrefabMarker);
                        }
                    }
                })
            );

        commands.entity(e).insert(SceneAutoRoot);

        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }
        if tr.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitivePrefab`]
pub fn sync_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitivePrefab), Changed<MeshPrimitivePrefab>>,
    mut meshs: ResMut<Assets<Mesh>>,
) {
    for (e, pref) in query.iter() {
        let mesh = meshs.add(pref.to_mesh());
        commands.entity(e).insert(mesh);
    }
}

/// System to sync [`StandardMaterial`] and [`MaterialPrefab`]
pub fn sync_material(
    mut commands: Commands,
    query: Query<(Entity, &MaterialPrefab), Changed<MaterialPrefab>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, pref) in query.iter() {
        let mat = materials.add(pref.to_material(&asset_server));
        commands.entity(e).insert(mat);
    }
}

/// remove mesh handle if prefab struct was removed in editor states
pub fn editor_remove_mesh(
    mut commands: Commands,
    mut query: RemovedComponents<MeshPrimitivePrefab>,
) {
    for e in query.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>();
            info!("Removed mesh handle for {:?}", e);
        }
    }
}

/// Spawn system on enter to [`EditorState::Game`] state
///
/// [`EditorState::Game`]: crate::EditorState::Game
pub fn spawn_player_start(
    mut commands: Commands,
    query: Query<(Entity, &PlayerStart)>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        info!("Spawning player start {:?} {}", e, &prefab.prefab);
        let child = commands
            .spawn(DynamicSceneBundle {
                scene: asset_server.load(prefab.prefab.to_string()),
                ..default()
            })
            .id();
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
