use bevy::prelude::*;

use space_editor_tabs::prelude::*;
use space_undo::ChangeChain;

use crate::editor_tab_name::EditorTabName;

pub struct ChangeChainViewPlugin;

impl Plugin for ChangeChainViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(
            ChangeChainView,
        );
    }
}

#[derive(Resource, Default)]
pub struct ChangeChainView;

impl EditorTab for ChangeChainView {
    fn ui(
        &mut self,
        ui: &mut bevy_egui::egui::Ui,
        _commands: &mut bevy::prelude::Commands,
        world: &mut bevy::prelude::World,
    ) {
        let change_chain = world.resource::<ChangeChain>();

        for change in change_chain.changes.iter() {
            ui.label(change.debug_text());
        }
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::ChangeChain.into()
    }
}
