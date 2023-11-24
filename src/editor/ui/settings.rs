use bevy::prelude::*;
use bevy_egui::*;

use crate::{
    editor::core::AllHotkeys,
    prelude::{EditorTab, EditorTabName},
};

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

#[derive(Default, Resource)]
pub struct SettingsWindow {
    read_input_for_hotkey: Option<String>,
}

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

        if world.contains_resource::<AllHotkeys>() {
            egui::Grid::new("hotkeys_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    world.resource_scope::<AllHotkeys, _>(|world, all_hotkeys| {
                        all_hotkeys.map(world, &mut |world, hotkey_name, bindings| {
                            ui.label(&hotkey_name);

                            if let Some(read_input_for_hotkey) = &self.read_input_for_hotkey {
                                if hotkey_name == *read_input_for_hotkey {
                                    let _ = ui.button("Wait for input");

                                    world.resource_scope::<Input<KeyCode>, _>(|_world, input| {
                                        for key in input.get_just_pressed() {
                                            bindings.clear();
                                            bindings.push(*key);
                                            self.read_input_for_hotkey = None;
                                        }
                                    });
                                } else {
                                    let binding_text =if bindings.len() == 1 {
                                        format!("{:?}", &bindings[0])
                                    } else {
                                        format!("{:?}", bindings)
                                    };

                                    if ui.button(binding_text).clicked() {
                                        self.read_input_for_hotkey = Some(hotkey_name);
                                    }
                                }
                            } else {
                                let binding_text = if bindings.len() == 1 {
                                    format!("{:?}", &bindings[0])
                                } else {
                                    format!("{:?}", bindings)
                                };

                                if ui.button(binding_text).clicked() {
                                    self.read_input_for_hotkey = Some(hotkey_name);
                                }
                            }

                            ui.end_row();
                        });
                    });
                });
        }

        // egui::Grid::new("hotkeys")
        //     .num_columns(2)
        //     .striped(true)
        //     .show(ui, |ui| {
        //         ui.label("Select object");
        //         ui.label("Left mouse button");
        //         ui.end_row();

        //         ui.label("Move/rotate/scale/clone \nmany objects simultaneously");
        //         ui.label("Shift");
        //         ui.end_row();

        //         ui.label("Clone object");
        //         ui.label("Alt");
        //         ui.end_row();

        //         ui.label("Delete object");
        //         ui.label("Delete or X");
        //         ui.end_row();
        //     });
    }

    fn title(&self) -> egui::WidgetText {
        "Settings".into()
    }
}
