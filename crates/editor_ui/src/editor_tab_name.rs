use space_editor_tabs::tab_name::TabName;

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum EditorTabName {
    CameraView,
    EventDispatcher,
    GameView,
    Hierarchy,
    Inspector,
    Resource,
    RuntimeAssets,
    Settings,
    ToolBox,
    ChangeChain,
    DebugWorldInspector,
}

impl TabName for EditorTabName {
    fn clear_background(&self) -> bool {
        *self != Self::GameView
    }

    fn title(&self) -> String {
        match self {
            Self::CameraView => "Camera View".to_string(),
            Self::EventDispatcher => "Event Dispatcher".to_string(),
            Self::GameView => "Game View".to_string(),
            Self::Hierarchy => "Hierarchy".to_string(),
            Self::Inspector => "Inspector".to_string(),
            Self::Resource => "Resource".to_string(),
            Self::RuntimeAssets => "Runtime Assets".to_string(),
            Self::Settings => "Settings".to_string(),
            Self::ToolBox => "Tool Box".to_string(),
            Self::ChangeChain => "Change Chain".to_string(),
            Self::DebugWorldInspector => "Debug World Inspector".to_string(),
        }
    }
}
