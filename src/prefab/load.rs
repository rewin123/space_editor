use bevy::{prelude::*, utils::HashMap};
use bevy_scene_hook::SceneHook;

use crate::{editor_registry::EditorRegistryExt, PrefabMarker};


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
        app.add_systems(Update, load_prefab);

    }
}

#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component)]
pub struct PrefabLoader {
    pub path : String
}




fn load_prefab(
    mut commands : Commands,
    query : Query<(Entity, &PrefabLoader), Changed<PrefabLoader>>,
    mut assets : ResMut<AssetServer>
) {
    for (e, l) in query.iter() {
        let scene : Handle<DynamicScene> = assets.load(&l.path);
        commands.entity(e).insert(scene)
            .insert(SceneHook::new(|e, cmd| {
                if e.contains::<Transform>() {
                    cmd.insert(Visibility::default());
                }
                if e.contains::<PrefabMarker>() {
                    cmd.remove::<PrefabMarker>();
                }
            }));
    }
}