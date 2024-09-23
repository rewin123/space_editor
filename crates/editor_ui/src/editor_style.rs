use bevy::prelude::*;
use space_editor_tabs::prelude::*;

use crate::{colors::*, sizing::Sizing};
use bevy_egui::egui::FontFamily::{Monospace, Proportional};
use bevy_egui::egui::{FontId, Rounding, TextStyle as ETextStyle};

#[derive(Resource, Default)]
pub struct EditorStyle {}

impl TabStyle for EditorStyle {
    fn error_color(&self) -> bevy_egui::egui::Color32 {
        ERROR_COLOR
    }

    fn set_egui_style(&self, _: &World, stl: &mut bevy_egui::egui::Style) {
        stl.spacing.button_padding = bevy_egui::egui::Vec2::new(8., 2.);
        stl.spacing.icon_spacing = 4.;
        stl.spacing.icon_width = 16.;
        stl.spacing.menu_margin = bevy_egui::egui::Margin {
            left: 8.,
            right: 8.,
            top: 4.,
            bottom: 8.,
        };
        stl.visuals.error_fg_color = ERROR_COLOR;
        stl.visuals.hyperlink_color = HYPERLINK_COLOR;
        stl.visuals.warn_fg_color = WARN_COLOR;
        stl.visuals.menu_rounding = Rounding::same(0.5);
        stl.text_styles = [
            (ETextStyle::Small, FontId::new(10.0, Proportional)),
            (ETextStyle::Body, FontId::new(12., Proportional)),
            (ETextStyle::Button, FontId::new(14., Proportional)),
            (ETextStyle::Heading, FontId::new(20.0, Proportional)),
            (ETextStyle::Monospace, FontId::new(12.0, Monospace)),
        ]
        .into()
    }

    fn text_size(&self, world: &World) -> f32 {
        world.get_resource::<Sizing>().map_or(12., |size| size.text)
    }
}
