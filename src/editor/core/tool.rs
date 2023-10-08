use bevy::prelude::*;

#[derive(Resource, Reflect, Clone, Debug)]
pub struct SelectedTool {
    pub tool: ToolName,
}

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

