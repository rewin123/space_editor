use bevy::prelude::*;

use super::{editor_tab::EditorTab, EditorUiAppExt};
use space_undo::ChangeChain;

pub struct ChangeChainViewPlugin;

impl Plugin for ChangeChainViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(
            super::editor_tab::EditorTabName::Other("Change Chain".to_string()),
            ChangeChainView,
        );
    }
}

#[derive(Resource, Default)]
pub struct ChangeChainView;

impl EditorTab for ChangeChainView {
    fn ui(
        &mut self,
        ui: &mut bevy_egui_next::egui::Ui,
        _commands: &mut bevy::prelude::Commands,
        world: &mut bevy::prelude::World,
    ) {
        let change_chain = world.resource::<ChangeChain>();

        for change in change_chain.changes.iter() {
            ui.label(change.debug_text());
        }
    }

    fn title(&self) -> bevy_egui_next::egui::WidgetText {
        "Change Chain".into()
    }
}
