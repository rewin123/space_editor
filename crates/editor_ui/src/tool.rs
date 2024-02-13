use bevy::prelude::*;

use crate::prelude::GameViewTab;

pub trait EditorTool {
    fn ui(&mut self, ui: &mut bevy_egui_next::egui::Ui, commands: &mut Commands, world: &mut World);
    fn name(&self) -> &str;
}

#[derive(Reflect, Clone, Debug, Default)]
pub enum ToolName {
    #[default]
    Gizmo,
    #[cfg(feature = "floor_plan")]
    FloorMap,
    Other(String),
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gizmo => "gizmo",
            #[cfg(feature = "floor_plan")]
            Self::FloorMap => "floor map",
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
        self.world
            .resource_mut::<GameViewTab>()
            .tools
            .push(Box::new(tool));
    }
}
