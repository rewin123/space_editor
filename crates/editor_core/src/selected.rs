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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_wireframes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, clear_wireframes);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Wireframe);
            commands.spawn(Wireframe);
        });
        app.update();

        let mut query = app.world.query_filtered::<Entity, With<Wireframe>>();
        assert_eq!(0, query.iter(&app.world).count());
    }

    #[test]
    fn removes_wireframe_if_not_selected() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, selected_entity_wireframe_update);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Wireframe);
            commands.spawn(Wireframe);
        });
        app.update();

        let mut query = app.world.query_filtered::<Entity, With<Wireframe>>();
        assert_eq!(0, query.iter(&app.world).count());
    }

    #[test]
    fn adds_wireframe_if_selected() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, selected_entity_wireframe_update);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Selected);
            commands.spawn(Selected);
        });
        app.update();

        let mut query = app
            .world
            .query_filtered::<Entity, (With<Wireframe>, With<Selected>)>();
        assert_eq!(2, query.iter(&app.world).count());
    }
}
