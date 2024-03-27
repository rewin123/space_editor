use bevy::prelude::*;

use space_editor_tabs::prelude::*;

use crate::editor_tab_name::EditorTabName;

pub struct SystemsViewPlugin;

impl Plugin for SystemsViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(SystemsView);
    }
}

#[derive(Resource, Default)]
pub struct SystemsView;

impl EditorTab for SystemsView {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _commands: &mut bevy::prelude::Commands,
        _world: &mut bevy::prelude::World,
    ) {
        ui.hyperlink_to("Cargo", "code -r . --goto \"./Cargo.toml:31:0\"");
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::Systems.into()
    }
}
