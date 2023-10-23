use bevy::prelude::*;

use crate::prelude::GameViewTab;

pub trait EditorTool {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, world: &mut World);
    fn name(&self) -> &str;
}

#[derive(Reflect, Clone, Debug)]
pub enum ToolName {
    Gizmo,
    #[cfg(feature = "floor_plan")]
    FloorMap,
    Other(String),
}

impl Default for ToolName {
    fn default() -> Self {
        ToolName::Gizmo
    }
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            ToolName::Gizmo => "gizmo",
            #[cfg(feature = "floor_plan")]
            ToolName::FloorMap => "floor map",
            ToolName::Other(name) => name,
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
