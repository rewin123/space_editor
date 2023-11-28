use bevy::{prelude::*, utils::HashSet};
use bevy_egui::*;

use crate::{
    editor::core::{AllHotkeys, ChangeChainSettings},
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
    all_pressed_hotkeys: HashSet<KeyCode>,
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

        ui.heading("Undo");
        world.resource_scope::<ChangeChainSettings, _>(|_world, mut settings| {
            ui.add(
                egui::DragValue::new(&mut settings.max_change_chain_size)
                    .prefix("Max change chain size: "),
            );
        });

        if world.contains_resource::<AllHotkeys>() {
            egui::Grid::new("hotkeys_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    world.resource_scope::<AllHotkeys, _>(|world, all_hotkeys| {
                        all_hotkeys.global_map(world, &mut |world, set| {
                            ui.heading(set.get_name());
                            ui.end_row();
                            let all_bindings = set.get_flat_bindings();
                            for (hotkey_name, bindings) in all_bindings {
                                ui.label(&hotkey_name);

                                if let Some(read_input_for_hotkey) = &self.read_input_for_hotkey {
                                    if hotkey_name == *read_input_for_hotkey {
                                        let mut key_text = String::new();

                                        world.resource_scope::<Input<KeyCode>, _>(
                                            |_world, input| {
                                                let all_pressed = input
                                                    .get_pressed()
                                                    .copied()
                                                    .collect::<Vec<_>>();
                                                self.all_pressed_hotkeys.extend(all_pressed.iter());
                                                let all_pressed = self
                                                    .all_pressed_hotkeys
                                                    .iter()
                                                    .copied()
                                                    .collect::<Vec<_>>();

                                                if all_pressed.is_empty() {
                                                    key_text = "Wait for input".to_string();
                                                } else {
                                                    key_text = format!("{:?}", all_pressed[0]);
                                                    for key in all_pressed.iter().skip(1) {
                                                        key_text =
                                                            format!("{} + {:?}", key_text, key);
                                                    }
                                                }

                                                if input.get_just_released().len() > 0 {
                                                    bindings.clear();
                                                    *bindings = all_pressed;
                                                    self.read_input_for_hotkey = None;
                                                    self.all_pressed_hotkeys.clear();
                                                }

                                                ui.add(egui::Button::new(
                                                    egui::RichText::new(&key_text).strong(),
                                                ));
                                            },
                                        );
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
                            }
                        });
                    });
                });
        }
    }

    fn title(&self) -> egui::WidgetText {
        "Settings".into()
    }
}
