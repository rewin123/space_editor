use bevy::prelude::*;
use bevy_scene_hook::SceneHook;

use crate::{editor_registry::EditorRegistryExt, PrefabMarker};

use super::save::ChildrenPrefab;


#[derive(Default, Bundle)]
pub struct PrefabBundle {
    loader : PrefabLoader,
    transform : Transform,
    global_transform : GlobalTransform,
    
    visiblity : Visibility,
    computed_visiblity : ComputedVisibility
}

impl PrefabBundle {
    pub fn new(path : &str) -> Self {
        Self {
            loader : PrefabLoader { path: path.to_string() },
            ..default()
        }
    }
}

pub struct LoadPlugin;

#[derive(Component)]
pub struct PrefabAutoChild;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<PrefabLoader>();
        app.add_systems(Update, load_prefab
            .after(bevy_scene_hook::Systems::SceneHookRunner));
        app.add_systems(Update, conflict_resolve
            .after(bevy_scene_hook::Systems::SceneHookRunner)
            .before(load_prefab));
        app.add_systems(Update, auto_children);

    }
}

#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component)]
pub struct PrefabLoader {
    pub path : String
}




fn load_prefab(
    mut commands : Commands,
    query : Query<(Entity, &PrefabLoader, Option<&Children>, Option<&Transform>, Option<&Visibility>), Changed<PrefabLoader>>,
    auto_childs : Query<Entity, With<PrefabAutoChild>>,
    mut assets : ResMut<AssetServer>
) {
    for (e, l, children, tr, vis) in query.iter() {

        if tr.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }

        //remove old scene
        if let Some(children) = children {
            for child in children {
                if auto_childs.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
            commands.entity(e).clear_children();
        }

        let scene : Handle<DynamicScene> = assets.load(&l.path);
        commands.entity(e).with_children(|cmds| {
            cmds.spawn(
                DynamicSceneBundle {
                    scene : scene,
                    ..default()
                }
            )
            .insert(SceneHook::new(|e, cmd| {
                cmd.insert(PrefabAutoChild);
            }))
            .insert(PrefabAutoChild);
        });
    }
}

fn conflict_resolve(
    mut commands : Commands,
    query : Query<Entity, (With<PrefabAutoChild>, With<PrefabMarker>)>
) {
    for e in query.iter() {
        commands.entity(e).remove::<PrefabMarker>();
    }
}

fn auto_children(
    mut commands : Commands,
    query : Query<(Entity, &ChildrenPrefab)>,
    existen_entity : Query<Entity>
) {
    for (e, children) in query.iter() {
        let mut cmds = commands.entity(e);
        for child in children.0.iter() {
            if existen_entity.contains(*child) {
                cmds.add_child(*child);
            }
        }
        cmds.remove::<ChildrenPrefab>();
    }
}