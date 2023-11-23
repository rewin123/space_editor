use bevy::prelude::*;

use crate::editor::core::ChangeChain;
use super::{editor_tab::EditorTab, EditorUiAppExt};

pub struct ChangeChainViewPlugin;

impl Plugin for ChangeChainViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(super::editor_tab::EditorTabName::Other("Change Chain".to_string()), ChangeChainView::default());
    }
}

#[derive(Resource)]
pub struct ChangeChainView {

}

impl Default for ChangeChainView {
    fn default() -> Self {
        Self {
        }
    }
}

impl EditorTab for ChangeChainView {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, commands: &mut bevy::prelude::Commands, world: &mut bevy::prelude::World) {
        let change_chain = world.resource::<ChangeChain>();

        for change in change_chain.changes.iter() {
            ui.label(change.debug_text());
        }
    }

    fn title(&self) -> bevy_egui::egui::WidgetText {
        "Change Chain".into()
    }
}