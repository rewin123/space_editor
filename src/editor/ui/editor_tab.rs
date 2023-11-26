// This module contains the implementation of the editor tabs

use bevy::{prelude::*, utils::HashMap};
use bevy_egui::egui::{self, WidgetText};

use super::{EditorUiRef, EditorUiReg};

pub trait EditorTab {
    fn ui(&mut self, ui: &mut egui::Ui, commands: &mut Commands, world: &mut World);
    fn title(&self) -> egui::WidgetText;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum EditorTabName {
    Hierarchy,
    GameView,
    Inspector,
    ToolBox,
    Settings,
    Other(String),
}

pub type EditorTabShowFn = Box<dyn Fn(&mut egui::Ui, &mut Commands, &mut World) + Send + Sync>;
pub type EditorTabGetTitleFn = Box<dyn Fn(&mut World) -> WidgetText + Send + Sync>;

pub enum EditorTabCommand {
    Add {
        name: EditorTabName,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
    },
}

pub struct EditorTabViewer<'a, 'w, 's> {
    pub world: &'a mut World,
    pub commands: &'a mut Commands<'w, 's>,
    pub registry: &'a mut HashMap<EditorTabName, EditorUiReg>,
    pub visible: Vec<EditorTabName>,
    pub tab_commands: Vec<EditorTabCommand>,
}

impl<'a, 'w, 's> egui_dock::TabViewer for EditorTabViewer<'a, 'w, 's> {
    type Tab = EditorTabName;

    fn ui(&mut self, ui: &mut egui::Ui, tab_name: &mut Self::Tab) {
        if let Some(reg) = self.registry.get_mut(tab_name) {
            match reg {
                EditorUiReg::ResourceBased {
                    show_command,
                    title_command: _,
                } => {
                    show_command(ui, self.commands, self.world);
                }
                EditorUiReg::Schedule => {
                    self.world.resource_scope(
                        |world, mut storage: Mut<ScheduleEditorTabStorage>| {
                            if let Some(tab) = storage.0.get_mut(tab_name) {
                                tab.ui(ui, self.commands, world);
                            } else {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    "Not implemented schedule tab",
                                );
                            }
                        },
                    );
                }
            }
        } else {
            ui.colored_label(egui::Color32::RED, "Not implemented panel");
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if let Some(reg) = self.registry.get_mut(tab) {
            match reg {
                EditorUiReg::ResourceBased {
                    show_command: _,
                    title_command,
                } => title_command(self.world),
                EditorUiReg::Schedule => self
                    .world
                    .resource_mut::<ScheduleEditorTabStorage>()
                    .0
                    .get(tab)
                    .map_or_else(|| format!("{tab:?}").into(), |tab| tab.title.clone()),
            }
        } else {
            format!("{tab:?}").into()
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EditorTabName::GameView)
    }

    fn add_popup(
        &mut self,
        ui: &mut egui::Ui,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
    ) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;
        let mut counter = 0;
        for reg in self.registry.iter() {
            if !self.visible.contains(reg.0) {
                let format_name;
                if let EditorTabName::Other(name) = reg.0 {
                    format_name = name.clone();
                } else {
                    format_name = format!("{:?}", reg.0);
                }

                if ui.button(format_name).clicked() {
                    self.tab_commands.push(EditorTabCommand::Add {
                        name: reg.0.clone(),
                        surface,
                        node,
                    });
                }
                counter += 1;
            }
        }

        if counter == 0 {
            ui.label("All tabs are showing");
        }
    }
}

pub struct ScheduleEditorTab {
    pub schedule: Schedule,
    pub title: egui::WidgetText,
}

impl EditorTab for ScheduleEditorTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        let inner_ui = ui.child_ui(ui.max_rect(), *ui.layout());
        world.insert_non_send_resource(EditorUiRef(inner_ui));

        self.schedule.run(world);
        world.remove_non_send_resource::<EditorUiRef>();
    }

    fn title(&self) -> egui::WidgetText {
        self.title.clone()
    }
}

#[derive(Resource, Default)]
pub struct ScheduleEditorTabStorage(pub HashMap<EditorTabName, ScheduleEditorTab>);
