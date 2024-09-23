use bevy::prelude::*;

use space_editor_tabs::prelude::*;
use space_undo::ChangeChain;

use crate::editor_tab_name::EditorTabName;

pub struct ChangeChainViewPlugin;

impl Plugin for ChangeChainViewPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(ChangeChainView);
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
        let Some(change_chain) = world.get_resource::<ChangeChain>() else {
            error!("Failed to get Change Chain");
            return;
        };

        for change in change_chain.changes.iter() {
            ui.label(change.debug_text());
        }
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::ChangeChain.into()
    }
}
