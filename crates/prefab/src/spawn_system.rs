use bevy::prelude::*;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};

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
        ),
        Changed<GltfPrefab>,
    >,
    auto_childs: Query<&SceneAutoChild>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab, children, vis, tr) in prefabs.iter() {
        if let Some(children) = children {
            for e in children {
                if auto_childs.contains(*e) {
                    commands.entity(*e).despawn_recursive();
                }
            }
        }

        let id = commands
            .spawn(HookedSceneBundle {
                scene: SceneBundle {
                    scene: asset_server.load(format!("{}#{}", &prefab.path, &prefab.scene)),
                    ..default()
                },
                hook: SceneHook::new(|_e, cmd| {
                    cmd.insert(SceneAutoChild);
                }),
            })
            .insert(SceneAutoChild)
            .id();
        commands.entity(e).add_child(id);

        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }
        if tr.is_none() {
            #[cfg(feature = "f32")]
            commands.entity(e).insert(TransformBundle::default());
            #[cfg(feature = "f64")]
            {
                commands
                    .entity(e)
                    .insert(bevy_transform64::DTransformBundle::default());
            }
        }
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitivePrefab`]
pub fn sync_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitivePrefab), Changed<MeshPrimitivePrefab>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, prefab) in query.iter() {
        let mesh = meshes.add(prefab.to_mesh());
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
    for (e, prefab) in query.iter() {
        let mat = materials.add(prefab.to_material(&asset_server));
        commands.entity(e).insert(mat);
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitive2dPrefab`]
pub fn sync_2d_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitive2dPrefab), Changed<MeshPrimitive2dPrefab>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, prefab) in query.iter() {
        let mesh = meshes.add(prefab.to_mesh());
        commands.entity(e).insert(mesh);
    }
}

/// System to sync [`ColorMaterial`] and [`ColorMaterialPrefab`]
pub fn sync_2d_material(
    mut commands: Commands,
    query: Query<(Entity, &ColorMaterialPrefab), Changed<ColorMaterialPrefab>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        let mat = materials.add(prefab.to_material(&asset_server));
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

/// remove mesh handle if prefab struct was removed in editor states
pub fn editor_remove_mesh_2d(
    mut commands: Commands,
    mut query: RemovedComponents<MeshPrimitive2dPrefab>,
) {
    for e in query.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>();
            info!("Removed mesh handle for {:?}", e);
        }
    }
}

/// System to sync [`SpriteBundle`] and [`SpriteTexture`]
pub fn sync_sprite_texture(
    mut commands: Commands,
    query: Query<(Entity, &SpriteTexture), Changed<SpriteTexture>>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        if let Some(sprite) = prefab.to_sprite(&asset_server) {
            commands.entity(e).insert(sprite);
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
