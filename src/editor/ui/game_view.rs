
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_egui::egui;

use crate::prelude::EditorTab;

#[derive(Default, Resource)]
pub struct GameViewTab {
    pub viewport_rect : Option<egui::Rect>
}

impl EditorTab for GameViewTab {
    fn ui(&mut self, ui : &mut bevy_egui::egui::Ui, world : &mut World) {
        self.viewport_rect = Some(ui.clip_rect());
    }

    fn title(&self) -> bevy_egui::egui::WidgetText {
        "Game view".into()
    }
}