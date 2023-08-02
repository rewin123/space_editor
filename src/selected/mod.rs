

use bevy::{prelude::*, utils::HashSet, pbr::wireframe::{Wireframe, WireframePlugin}};

#[derive(Resource, Default, Clone)]
pub struct SelectedEntities {
    pub list : HashSet<Entity>
}

pub struct SelectedPlugin;

impl Plugin for SelectedPlugin {
    fn build(&self, app : &mut App) {
        app.init_resource::<SelectedEntities>();
 
        if !app.is_plugin_added::<WireframePlugin>() {
            app.add_plugins(WireframePlugin);
        }
        app.add_systems(Update, stupid_wireframe_update);
    }
}

fn stupid_wireframe_update(
    mut cmds : Commands,
    mut query : Query<(Entity, &Wireframe)>,
    selected : Res<SelectedEntities>
) {
    for (e, w) in query.iter() {
        if !selected.list.contains(&e) {
            cmds.entity(e).remove::<Wireframe>();
        }
    }

    for e in &selected.list {
        cmds.entity(*e).insert(Wireframe);
    }
}