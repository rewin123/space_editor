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
    DebugWorldInspector
}

impl TabName for EditorTabName {
    fn clear_background(&self) -> bool {
        if *self == EditorTabName::GameView {
            false
        } else {
            true
        }
    }

    fn title(&self) -> String {
        match self {
            EditorTabName::CameraView => "Camera View".to_string(),
            EditorTabName::EventDispatcher => "Event Dispatcher".to_string(),
            EditorTabName::GameView => "Game View".to_string(),
            EditorTabName::Hierarchy => "Hierarchy".to_string(),
            EditorTabName::Inspector => "Inspector".to_string(),
            EditorTabName::Resource => "Resource".to_string(),
            EditorTabName::RuntimeAssets => "Runtime Assets".to_string(),
            EditorTabName::Settings => "Settings".to_string(),
            EditorTabName::ToolBox => "Tool Box".to_string(),
            EditorTabName::ChangeChain => "Change Chain".to_string(),
            EditorTabName::DebugWorldInspector => "Debug World Inspector".to_string(),
        }
    }
}
