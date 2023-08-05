pub mod component;
pub mod spawn_system;
pub mod save;
pub mod load;

use bevy::prelude::*;
use bevy_scene_hook::HookPlugin;

use crate::{editor::prelude::InspectorPlugin, editor_registry::EditorRegistryExt, prelude::{EditorRegistry, EditorRegistryPlugin}};

use component::*;
use spawn_system::*;
use save::*;
use load::*;

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HookPlugin>() {
            app.add_plugins(HookPlugin);
        }

        if !app.is_plugin_added::<EditorRegistryPlugin>() {
            app.add_plugins(EditorRegistryPlugin);
        }
        
        app.editor_registry::<GltfPrefab>();

        app.add_systems(Update, spawn_scene);
        app.add_systems(Update, (add_global_transform, remove_global_transform, add_computed_visiblity, remove_computed_visiblity));

        app.add_plugins(SavePrefabPlugin);
        app.add_plugins(LoadPlugin);

    }
}


pub fn add_global_transform(
    mut commands : Commands,
    mut query : Query<(Entity, &mut Transform, Option<&Parent>), (With<Transform>, Without<GlobalTransform>)>,
    mut globals : Query<&GlobalTransform>
) {
    for (e, mut tr, parent) in query.iter_mut() {
        if let Some(parent) = parent {
            if let Ok(parent_global) = globals.get(parent.get()) {
                commands.entity(e).insert(parent_global.mul_transform(tr.clone()));
            } else {
                commands.entity(e).insert(GlobalTransform::from(*tr));
            }
        } else {
            commands.entity(e).insert(GlobalTransform::from(*tr));

        }
        tr.set_changed();
    }
}

fn remove_global_transform(
    mut commands : Commands,
    query : Query<Entity, (Without<Transform>, With<GlobalTransform>)>
) {
    for e in query.iter() {
        commands.entity(e).remove::<GlobalTransform>();
    }
}

fn add_computed_visiblity(
    mut commands : Commands,
    query : Query<Entity, (With<Visibility>, Without<ComputedVisibility>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(ComputedVisibility::default());
    }
}

fn remove_computed_visiblity(
    mut commands : Commands,
    query : Query<Entity, (Without<Visibility>, With<ComputedVisibility>)>
) {
    for e in query.iter() {
        commands.entity(e).remove::<ComputedVisibility>();
    }
}