use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
};

use space_shared::{EditorSet, EditorState};

/// A marker for editor selected entities
#[derive(Component, Default, Clone)]
pub struct Selected;

/// Selection system plugins
pub struct SelectedPlugin;

impl Plugin for SelectedPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<WireframePlugin>() {
            app.add_plugins(WireframePlugin);
        }
        app.add_systems(
            Update,
            selected_entity_wireframe_update.in_set(EditorSet::Editor),
        );
        app.add_systems(OnEnter(EditorState::GamePrepare), clear_wireframes);
    }
}

fn selected_entity_wireframe_update(
    mut cmds: Commands,
    del_wireframe: Query<Entity, (With<Wireframe>, Without<Selected>)>,
    need_wireframe: Query<Entity, (Without<Wireframe>, With<Selected>)>,
) {
    for e in del_wireframe.iter() {
        cmds.entity(e).remove::<Wireframe>();
    }

    for e in need_wireframe.iter() {
        cmds.entity(e).insert(Wireframe);
    }
}

fn clear_wireframes(mut cmds: Commands, del_wireframe: Query<Entity, With<Wireframe>>) {
    for e in del_wireframe.iter() {
        cmds.entity(e).remove::<Wireframe>();
    }
}
