//code only for editor gui

use bevy::prelude::*;

pub mod inspector;
pub mod bot_menu;
pub mod hierarchy;
pub mod selected;
pub mod asset_insector;

pub mod prelude {
    pub use super::inspector::*;
    pub use super::bot_menu::*;
    pub use super::hierarchy::*;
    pub use super::selected::*;
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }
        
        app.add_plugins(prelude::SpaceHierarchyPlugin::default())
        .add_plugins(prelude::InspectorPlugin)
        .add_plugins(prelude::BotMenuPlugin);
    }
}