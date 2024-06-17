use crate::{tab_name::TabNameHolder, NewTabBehaviour};
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
    fn tab_name(&self) -> TabNameHolder;
}
