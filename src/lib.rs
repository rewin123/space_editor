pub mod hierarchy; 
pub mod selected;
pub mod inspector;
pub mod asset_insector;
pub mod prefab;
pub mod top_menu;

use bevy::prelude::*;

pub struct SpaceEditorPlugin {

}


impl Default for SpaceEditorPlugin {
    fn default() -> Self {
        Self {

        }
    }
}

impl Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin);
        app.add_plugins(selected::SelectedPlugin);
        app.add_plugins(hierarchy::SpaceHierarchyPlugin::default());
        app.add_plugins(top_menu::TopMenuPlugin);
    }
}


//editor shows only entities with this marker
#[derive(Component)]
pub struct PrefabMarker;
