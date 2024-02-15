use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_egui_next::*;
use space_editor_core::hotkeys::AllHotkeys;
use space_undo::ChangeChainSettings;

#[cfg(feature = "persistence_editor")]
use space_persistence::*;

use super::{
    editor_tab::{EditorTab, EditorTabName},
    EditorUiAppExt,
};

const TAB_MODES: [NewTabBehaviour; 3] = [
    NewTabBehaviour::Pop,
    NewTabBehaviour::SameNode,
    NewTabBehaviour::SplitNode,
];

const GAME_MODES: [GameMode; 2] = [GameMode::Game2D, GameMode::Game3D];

pub struct SettingsWindowPlugin;

impl Plugin for SettingsWindowPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::Settings, SettingsWindow::default());
        app.register_type::<GameMode>()
            .init_resource::<GameModeSettings>();
        #[cfg(feature = "persistence_editor")]
        {
            app.persistence_resource::<NewWindowSettings>()
                .register_type::<NewTabBehaviour>()
                .init_resource::<NewWindowSettings>();
            app.persistence_resource::<ChangeChainSettings>();
            app.persistence_resource::<GameModeSettings>();
        }

        // #[cfg(feature = "bevy_xpbd_3d")]
        // {
        //     #[cfg(feature = "persistence_editor")]
        //     {
        //         app.persistence_resource::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>();
        //         app.register_type::<Option<Vec3>>();
        //         app.register_type::<Option<Color>>();
        //         app.register_type::<Option<[f32; 4]>>();
        //         app.register_type::<[f32; 4]>();
        //     }
        // }
    }
}

#[derive(Default, Reflect, PartialEq, Eq, Clone)]
pub enum GameMode {
    Game2D,
    #[default]
    Game3D,
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            Self::Game2D => String::from("2D"),
            Self::Game3D => String::from("3D"),
        }
    }
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GameModeSettings {
    pub mode: GameMode,
}

impl GameModeSettings {
    pub fn is_3d(&self) -> bool {
        self.mode == GameMode::Game3D
    }

    pub fn is_2d(&self) -> bool {
        self.mode == GameMode::Game2D
    }

    pub const MODE_2D: Self = Self {
        mode: GameMode::Game2D,
    };
    pub const MODE_3D: Self = Self {
        mode: GameMode::Game3D,
    };

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Game Mode");
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.spacing();
            egui::ComboBox::new("game_mode", "")
                .selected_text(self.mode.to_string())
                .show_ui(ui, |ui| {
                    for mode in GAME_MODES.into_iter() {
                        if ui
                            .selectable_label(self.mode == mode, mode.to_string())
                            .clicked()
                        {
                            self.mode = mode;
                        }
                    }
                });
        });
        ui.spacing();
        ui.separator();
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

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct NewWindowSettings {
    pub new_tab: NewTabBehaviour,
}

impl NewWindowSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::new("new_tab", "")
            .selected_text(self.new_tab.to_string())
            .show_ui(ui, |ui| {
                for mode in TAB_MODES.into_iter() {
                    if ui
                        .selectable_label(self.new_tab == mode, mode.to_string())
                        .clicked()
                    {
                        self.new_tab = mode;
                    }
                }
            });
    }
}

#[derive(Default, Resource)]
pub struct SettingsWindow {
    read_input_for_hotkey: Option<String>,
    all_pressed_hotkeys: HashSet<KeyCode>,
    sub_blocks: HashMap<
        String,
        Box<dyn FnMut(&mut egui::Ui, &mut Commands, &mut World) + Send + Sync + 'static>,
    >,
}

/// Trait for registering blocks in settings tab
pub trait RegisterSettingsBlockExt {
    /// Register ui block in settings tab
    fn register_settings_block(
        &mut self,
        name: &str,
        block: impl FnMut(&mut egui::Ui, &mut Commands, &mut World) + Send + Sync + 'static,
    );
}

impl RegisterSettingsBlockExt for App {
    fn register_settings_block(
        &mut self,
        name: &str,
        block: impl FnMut(&mut egui::Ui, &mut Commands, &mut World) + Send + Sync + 'static,
    ) {
        self.world
            .resource_mut::<SettingsWindow>()
            .sub_blocks
            .insert(name.to_string(), Box::new(block));
    }
}

impl EditorTab for SettingsWindow {
    fn ui(&mut self, ui: &mut egui::Ui, commands: &mut Commands, world: &mut World) {
        let game_mode_setting = &mut world.resource_mut::<GameModeSettings>();
        game_mode_setting.ui(ui);
        ui.spacing();

        ui.heading("Undo");
        world.resource_scope::<ChangeChainSettings, _>(|_world, mut settings| {
            ui.add(
                egui::DragValue::new(&mut settings.max_change_chain_size)
                    .prefix("Max change chain size: "),
            );
        });

        ui.spacing();
        ui.heading("New Tab Behaviour");
        let new_window_settings = &mut world.resource_mut::<NewWindowSettings>();
        new_window_settings.ui(ui);

        ui.spacing();
        ui.heading("Hotkeys in Game view tab");
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

            for (name, block) in self.sub_blocks.iter_mut() {
                ui.heading(name);
                (*block)(ui, commands, world);
            }
        }
    }

    fn title(&self) -> egui::WidgetText {
        "Settings".into()
    }
}
