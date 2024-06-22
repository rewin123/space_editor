/// Colors used in editor
use bevy_egui::egui::{Color32, Stroke};

pub fn stroke_default_color() -> Stroke {
    Stroke::new(1., STROKE_COLOR)
}
pub const STROKE_COLOR: Color32 = Color32::from_rgb(70, 70, 70);
pub const SPECIAL_BG_COLOR: Color32 = Color32::from_rgb(20, 20, 20);
pub const DEFAULT_BG_COLOR: Color32 = Color32::from_rgb(27, 27, 27);
pub const PLAY_COLOR: Color32 = Color32::from_rgb(0, 194, 149);
pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 59, 33);
pub const HYPERLINK_COLOR: Color32 = Color32::from_rgb(99, 235, 231);
pub const WARN_COLOR: Color32 = Color32::from_rgb(225, 206, 67);
pub const SELECTED_ITEM_COLOR: Color32 = Color32::from_rgb(76, 93, 235);
pub const TEXT_COLOR: Color32 = Color32::WHITE;
