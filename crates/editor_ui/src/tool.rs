use bevy::prelude::*;

use crate::prelude::GameViewTab;

pub trait EditorTool {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, commands: &mut Commands, world: &mut World);
    fn name(&self) -> &str;
}

#[derive(Reflect, Clone, Debug, Default)]
pub enum ToolName {
    #[default]
    Gizmo,
    Other(String),
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gizmo => "gizmo",
            Self::Other(name) => name,
        }
    }
}

pub trait ToolExt {
    fn editor_tool<T>(&mut self, tool: T)
    where
        T: EditorTool + Send + Sync + 'static;
}

impl ToolExt for App {
    fn editor_tool<T>(&mut self, tool: T)
    where
        T: EditorTool + Send + Sync + 'static,
    {
        if let Some(mut game_view) = self.world_mut().get_resource_mut::<GameViewTab>() {
            game_view.tools.push(Box::new(tool));
        } else {
            error!("Game View tab not loaded");
        }
    }
}
