use bevy::prelude::*;
use bevy_egui::*;

use crate::prelude::{EditorTab, EditorTabName};

use super::EditorUiAppExt;

pub struct SettingsWindowPlugin;

impl Plugin for SettingsWindowPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::Settings, SettingsWindow::default());
    }
}

#[derive(Default, Resource)]
pub struct SettingsWindow {}

impl EditorTab for SettingsWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _commands: &mut Commands, world: &mut World) {
        #[cfg(feature = "bevy_xpbd_3d")]
        {
            ui.heading("Bevy XPBD 3D");
            ui.checkbox(
                &mut world
                    .resource_mut::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>()
                    .enabled,
                "Show bevy xpbd debug render",
            );
            ui.checkbox(
                &mut world
                    .resource_mut::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>()
                    .hide_meshes,
                "Hide debug meshes",
            );
        }

        ui.heading("Hotkeys in Game view tab");

        egui::Grid::new("hotkeys")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Select object");
                ui.label("Left mouse button");
                ui.end_row();

                ui.label("Move object");
                ui.label("G");
                ui.end_row();

                ui.label("Rotate object");
                ui.label("R");
                ui.end_row();

                ui.label("Scale object");
                ui.label("S");
                ui.end_row();

                ui.label("Move/rotate/scale/clone \nmany objects simultaneously");
                ui.label("Shift");
                ui.end_row();

                ui.label("Clone object");
                ui.label("Alt");
                ui.end_row();

                ui.label("Delete object");
                ui.label("Delete or X");
                ui.end_row();
            });
    }

    fn title(&self) -> egui::WidgetText {
        "Settings".into()
    }
}
