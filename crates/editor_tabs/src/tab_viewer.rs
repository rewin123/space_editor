use crate::{
    prelude::{to_label, Sizing},
    schedule_editor_tab::ScheduleEditorTabStorage,
    tab_name::TabNameHolder,
    EditorTab, EditorUiReg, ERROR_COLOR,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::egui;
use convert_case::{Case, Casing};

pub enum EditorTabCommand {
    Add {
        name: TabNameHolder,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
    },
}

pub struct EditorTabViewer<'a, 'w, 's> {
    pub world: &'a mut World,
    pub commands: &'a mut Commands<'w, 's>,
    pub registry: &'a mut HashMap<TabNameHolder, EditorUiReg>,
    pub visible: Vec<TabNameHolder>,
    pub tab_commands: Vec<EditorTabCommand>,
}

impl<'a, 'w, 's> egui_dock::TabViewer for EditorTabViewer<'a, 'w, 's> {
    type Tab = TabNameHolder;

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
                                ui.colored_label(ERROR_COLOR, "Not implemented schedule tab");
                            }
                        },
                    );
                }
            }
        } else {
            ui.colored_label(ERROR_COLOR, "Not implemented panel");
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let sizing = self.world.resource::<Sizing>().clone();
        if let Some(reg) = self.registry.get(tab) {
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
                    .map_or_else(
                        || to_label(&format!("{tab:?}"), sizing.text).into(),
                        |tab| to_label(&tab.tab_name.title, sizing.text).into(),
                    ),
            }
        } else {
            to_label(&format!("{tab:?}"), sizing.text).into()
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        window.clear_background
    }

    fn add_popup(
        &mut self,
        ui: &mut egui::Ui,
        surface: egui_dock::SurfaceIndex,
        node: egui_dock::NodeIndex,
    ) {
        ui.set_min_width(200.0);
        ui.style_mut().visuals.button_frame = false;
        let mut counter = 0;
        let mut tab_registry: Vec<(&TabNameHolder, &EditorUiReg)> = self.registry.iter().collect();
        tab_registry.sort_by(|a, b| a.0.cmp(b.0));

        for registry in tab_registry.iter() {
            if !self.visible.contains(registry.0) {
                let format_name = format!("{}", registry.0.title)
                    .from_case(Case::Pascal)
                    .to_case(Case::Title);

                if ui.button(format_name).clicked() {
                    self.tab_commands.push(EditorTabCommand::Add {
                        name: registry.0.clone(),
                        surface,
                        node,
                    });
                }
                ui.spacing();
                counter += 1;
            }
        }

        if counter == 0 {
            ui.label("All tabs are showing");
        }
    }
}
