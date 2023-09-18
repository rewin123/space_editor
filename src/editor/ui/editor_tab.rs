use std::{any::TypeId, sync::Arc};

use bevy_egui::egui::{self, WidgetText};
use bevy::{prelude::*, utils::HashMap};

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
    pub show_commands : &'a mut HashMap<EditorTabName, EditorTabShowFn>,
    pub title_commands : &'a mut HashMap<EditorTabName, EditorTabGetTitleFn>
}

impl<'a> egui_dock::TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTabName;

    fn ui(&mut self, ui: &mut egui::Ui, tab_name: &mut Self::Tab) {
        if let Some(cmd) = self.show_commands.get_mut(tab_name) {
            cmd(ui, self.world);
        } else {
            ui.colored_label(egui::Color32::RED, "Not implemented panel");
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if let Some(cmd) = self.title_commands.get(tab) {
            cmd(self.world)
        } else {
            format!("{tab:?}").into()
        }
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EditorTabName::GameView)
    }
}