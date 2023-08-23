pub mod component;
pub mod spawn_system;
pub mod save;
pub mod load;

use bevy::prelude::*;
use bevy_scene_hook::HookPlugin;

use crate::{editor_registry::EditorRegistryExt, prelude::EditorRegistryPlugin, EditorState, EditorSet};

use component::*;
use spawn_system::*;
use save::*;
use load::*;

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {

        app.add_state::<EditorState>();

        if !app.is_plugin_added::<HookPlugin>() {
            app.add_plugins(HookPlugin);
        }

        if !app.is_plugin_added::<EditorRegistryPlugin>() {
            app.add_plugins(EditorRegistryPlugin);
        }

        app.configure_set(Update, EditorSet::Game.run_if(in_state(EditorState::Game)));
        app.configure_set(Update, EditorSet::Editor.run_if(in_state(EditorState::Editor)));
        
        app.editor_registry::<GltfPrefab>();
        app.editor_registry::<MaterialPrefab>();
        app.editor_registry::<MeshPrimitivePrefab>();

        app.editor_relation::<MeshPrimitivePrefab, Transform>();
        app.editor_relation::<MeshPrimitivePrefab, Visibility>();
        app.editor_relation::<MeshPrimitivePrefab, MaterialPrefab>();


        app.register_type::<SpherePrefab>();
        app.register_type::<BoxPrefab>();
        

        app.add_systems(Update, spawn_scene);
        app.add_systems(Update, (add_global_transform, remove_global_transform, add_computed_visiblity, remove_computed_visiblity));

        app.add_systems(
            Update,
            (sync_mesh, sync_material)
        );

        app.add_systems(
            Update,
            (editor_remove_mesh).run_if(in_state(EditorState::Editor))
        );

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