use crate::NewTabBehaviour;
/// This module contains the implementation of the editor tabs
use bevy::prelude::*;
use bevy_egui::egui;

pub const TAB_MODES: [NewTabBehaviour; 3] = [
    NewTabBehaviour::Pop,
    NewTabBehaviour::SameNode,
    NewTabBehaviour::SplitNode,
];

pub trait EditorTab {
    fn ui(&mut self, ui: &mut egui::Ui, commands: &mut Commands, world: &mut World);
    fn title(&self) -> egui::WidgetText;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum EditorTabName {
    CameraView,
    EventDispatcher,
    GameView,
    Hierarchy,
    Inspector,
    Resource,
    RuntimeAssets,
    Settings,
    ToolBox,
    Other(String),
}
