pub mod voxel;

pub mod debug;

use bevy::prelude::*;
use space_editor_ui::prelude::*;

pub struct VoxelEditorPlugin;

impl Plugin for VoxelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::Other("Voxel".to_string()), VoxelTab::default());
    }
}

#[derive(Resource, Default)]
pub struct VoxelTab {

}

impl EditorTab for VoxelTab {
    fn ui(&mut self, ui: &mut ext::bevy_inspector_egui::egui::Ui, commands: &mut Commands, world: &mut World) {
        
    }

    fn title(&self) -> ext::bevy_inspector_egui::egui::WidgetText {
        "Voxel".into()
    }
}
