use std::{any::TypeId, sync::Arc};

use bevy_egui::egui::{self, WidgetText};
use bevy::{prelude::*, utils::HashMap};

use super::{EditorUiReg, EditorUiRef};

pub trait EditorTab {
    fn ui(&mut self, ui : &mut egui::Ui, world : &mut World);
    fn title(&self) -> egui::WidgetText;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum EditorTabName {
    Hierarchy,
    GameView,
    Inspector,
    ToolBox,
    Other(String)
}

pub type EditorTabShowFn = Box<dyn Fn(&mut egui::Ui, &mut World) + Send + Sync>;
pub type EditorTabGetTitleFn = Box<dyn Fn(&mut World) -> WidgetText + Send + Sync>;

pub struct EditorTabViewer<'a> {
    pub world : &'a mut World,
    pub registry : &'a mut HashMap<EditorTabName, EditorUiReg>
}

impl<'a> egui_dock::TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTabName;

    fn ui(&mut self, ui: &mut egui::Ui, tab_name: &mut Self::Tab) {
        if let Some(reg) = self.registry.get_mut(tab_name) {
            match reg {
                EditorUiReg::Trait { show_command, title_command } => {
                    show_command(ui, self.world);
                },
                EditorUiReg::Schedule => {
                    self.world.resource_scope(|world, mut storage : Mut<ScheduleEditorTabStorage>| {
                        if let Some(tab) = storage.0.get_mut(tab_name) {
                            tab.ui(ui, world);
                        } else {
                            ui.colored_label(
                                egui::Color32::RED, 
                                "Not implemented schedule tab");
                        }
                    });
                    
                },
            }
            
        } else {
            ui.colored_label(egui::Color32::RED, "Not implemented panel");
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if let Some(reg) = self.registry.get_mut(tab) {
            match reg {
                EditorUiReg::Trait { show_command, title_command } => {
                    title_command(self.world)
                },
                EditorUiReg::Schedule => {
                    if let Some(tab) = self.world.resource_mut::<ScheduleEditorTabStorage>().0.get(tab) {
                        tab.title.clone()
                    } else {
                        format!("{tab:?}").into()
                    }
                },
            }
            
        } else {
            format!("{tab:?}").into()
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EditorTabName::GameView)
    }
}


pub struct ScheduleEditorTab {
    pub schedule : Schedule,
    pub  title : egui::WidgetText
}

impl EditorTab for ScheduleEditorTab {
    fn ui(&mut self, ui : &mut egui::Ui, world : &mut World) {
        let inner_ui = ui.child_ui(ui.max_rect(), ui.layout().clone());
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

