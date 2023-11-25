use bevy::prelude::*;
use bevy_egui::*;

use crate::prelude::{EditorTab, EditorTabName};

#[cfg(feature = "persistance_editor")]
use crate::prelude::editor::core::AppPersistanceExt;

use super::EditorUiAppExt;

pub struct SettingsWindowPlugin;

impl Plugin for SettingsWindowPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::Settings, SettingsWindow::default());

        #[cfg(feature = "bevy_xpbd_3d")]
        {
            #[cfg(feature = "persistance_editor")]
            {
                app.persistance_resource::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>();
                app.register_type::<Option<Vec3>>();
                app.register_type::<Option<Color>>();
                app.register_type::<Option<[f32; 4]>>();
                app.register_type::<[f32; 4]>();
            }
        }
    }
}

#[derive(Default, Reflect, PartialEq, Eq, Clone)]
pub enum NewTabBehaviour {
    Pop,
    #[default]
    SameNode,
    SplitNode,
}

impl ToString for NewTabBehaviour {
    fn to_string(&self) -> String {
        match self {
            Self::Pop => "New window",
            Self::SameNode => "Same Node",
            Self::SplitNode => "Splits Node",
        }
        .to_string()
    }
}

#[derive(Default, Resource, Clone)]
pub struct SettingsWindow {
    pub new_tab: NewTabBehaviour,
}

impl EditorTab for SettingsWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _commands: &mut Commands, world: &mut World) {
        let tab_modes: Vec<NewTabBehaviour> = vec![
            NewTabBehaviour::Pop,
            NewTabBehaviour::SameNode,
            NewTabBehaviour::SplitNode,
        ];

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

        ui.spacing();
        ui.heading("New Tab Behaviour");
        egui::ComboBox::new("new_tab", "")
            .selected_text(self.new_tab.to_string())
            .show_ui(ui, |ui| {
                for (_, mode) in tab_modes.into_iter().enumerate() {
                    if ui
                        .selectable_label(self.new_tab == mode, mode.to_string())
                        .clicked()
                    {
                        self.new_tab = mode;
                    }
                }
            });

        ui.spacing();
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
