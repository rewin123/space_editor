use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use egui_dock::egui::{Color32, RichText};

#[derive(Resource, Clone, PartialEq, Reflect, InspectorOptions, Debug)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct Sizing {
    pub icon: IconSize,
    pub gizmos: IconSize,
    #[inspector(min = 12.0, max = 24.0)]
    pub text: f32,
}

impl Default for Sizing {
    fn default() -> Self {
        Self {
            icon: IconSize::Regular,
            gizmos: IconSize::Gizmos,
            text: 14.,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Reflect, Debug)]
#[reflect(Default)]
pub enum IconSize {
    XSmall,
    Small,
    SmallPlus,
    Gizmos,
    #[default]
    Regular,
    Medium,
    Large,
    XLarge,
}

impl IconSize {
    pub const fn to_size(&self) -> f32 {
        match self {
            Self::XSmall => 12.,
            Self::Small => 16.,
            Self::SmallPlus => 18.,
            Self::Gizmos => 20.,
            Self::Regular => 20.,
            Self::Medium => 24.,
            Self::Large => 28.,
            Self::XLarge => 32.,
        }
    }
}

pub fn to_richtext(text: &str, size: &IconSize) -> RichText {
    RichText::new(text).size(size.to_size())
}

pub fn to_colored_richtext(text: &str, size: &IconSize, color: Color32) -> RichText {
    RichText::new(text).size(size.to_size()).color(color)
}

pub fn to_label(text: &str, size: f32) -> RichText {
    RichText::new(text)
        .size(size)
        .family(egui_dock::egui::FontFamily::Proportional)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assert_default_sizing() {
        let sizing = Sizing::default();

        let expected = Sizing {
            icon: IconSize::Regular,
            gizmos: IconSize::Gizmos,
            text: 14.,
        };

        assert_eq!(sizing, expected);
    }

    #[test]
    fn test_labels() {
        let label = to_label("text", 14.);
        assert_eq!(label.text(), "text");

        let text = to_richtext("test", &IconSize::Large);
        assert_eq!(text.text(), "test");

        let color_text = to_colored_richtext("test", &IconSize::Large, Color32::BLACK);
        assert!(!color_text.is_empty());
    }
}
