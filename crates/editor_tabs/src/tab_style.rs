use bevy::prelude::*;
use bevy_egui::egui::{self, RichText};

pub const DEFAULT_STYLE: DefaultStyle = DefaultStyle;

/// This trait use to get tab style
pub trait TabStyle: Resource {
    fn error_color(&self) -> egui::Color32;
    fn set_egui_style(&self, world: &World, style: &mut egui::Style);
    fn text_size(&self, world: &World) -> f32;

    fn collect_style(&self, world: &World) -> CollectedStyle {
        let error_color = self.error_color();
        let text_size = self.text_size(world);
        CollectedStyle {
            error_color,
            text_size,
        }
    }
}

pub struct CollectedStyle {
    pub error_color: egui::Color32,
    pub text_size: f32,
}

impl Default for CollectedStyle {
    fn default() -> Self {
        Self {
            error_color: egui::Color32::RED,
            text_size: 14.0,
        }
    }
}

pub fn to_label(text: &str, size: f32) -> RichText {
    RichText::new(text)
        .size(size)
        .family(egui_dock::egui::FontFamily::Proportional)
}

#[derive(Default, Resource)]
pub struct DefaultStyle;

impl TabStyle for DefaultStyle {
    fn error_color(&self) -> egui::Color32 {
        egui::Color32::RED
    }

    fn set_egui_style(&self, _world: &World, _style: &mut egui::Style) {}

    fn text_size(&self, _world: &World) -> f32 {
        14.0
    }
}
