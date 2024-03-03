pub mod voxel;

pub mod debug;

use bevy::prelude::*;
use space_editor_ui::prelude::*;

pub struct VoxelEditorPlugin;

impl Plugin for VoxelEditorPlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins((
            voxel::VoxelWorldPlugin,
        ));

        app.editor_tab_by_trait(EditorTabName::Other("Voxel".to_string()), VoxelTab::default());

        app.editor_registry::<voxel::terrain::TerrainViewer>();
    }
}

#[derive(Resource, Default)]
pub struct VoxelTab {

}

impl EditorTab for VoxelTab {
    fn ui(&mut self, ui: &mut ext::bevy_inspector_egui::egui::Ui, commands: &mut Commands, world: &mut World) {
        if ui.button("Enable voxel terrain").clicked() {
            let mut q_cameras = world.query_filtered::<Entity, (With<Camera3d>, With<EditorCameraMarker>)>();
            for camera in q_cameras.iter(world) {
                commands
                    .entity(camera)
                    .insert(voxel::terrain::TerrainViewer);
            }
        }
    }

    fn title(&self) -> ext::bevy_inspector_egui::egui::WidgetText {
        "Voxel".into()
    }
}
