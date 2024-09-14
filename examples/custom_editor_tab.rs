use bevy::prelude::*;
use space_editor::prelude::*;

/// This example shows how to create custom editor tabs
/// space_editor allows to create tabs by implementing trait EditorTab
/// or by using system based tabs

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        // Add trait based tab
        .editor_tab_by_trait(TraitEditorTab)
        // Add system based tab
        .editor_tab(CustomTabName::SystemBased, system_tab)
        // Add trait based tab as first tab in Bottom panel
        .layout_push_front::<DoublePanelGroup, _, _>(
            DoublePanel::BottomLeft,
            CustomTabName::TraitBased,
        )
        // Add system based tab as first tab in Main panel
        .layout_push_front::<DoublePanelGroup, _, _>(DoublePanel::Main, CustomTabName::SystemBased)
        .run();
}

#[derive(Debug)]
/// A custom tab name
enum CustomTabName {
    /// A trait based tab
    TraitBased,
    /// A system based tab
    SystemBased,
}

impl TabName for CustomTabName {
    /// Set clear_background to true to clear background of the panel
    fn clear_background(&self) -> bool {
        true
    }

    /// Return title of the tab
    fn title(&self) -> String {
        match self {
            Self::TraitBased => String::from("Trait Based"),
            Self::SystemBased => String::from("System Based"),
        }
    }
}

#[derive(Resource)]
/// A struct that implements EditorTab trait
/// which allows to create custom tabs in the editor
struct TraitEditorTab;

impl EditorTab for TraitEditorTab {
    /// This function is called when tab needs to be rendered
    /// ui is bevy_egui::egui::Ui and it allows to build ui inside the tab
    fn ui(
        &mut self,
        ui: &mut space_prefab::prelude::ext::bevy_inspector_egui::egui::Ui,
        _commands: &mut Commands,
        _world: &mut World,
    ) {
        ui.label("Trait Based");
    }

    /// Return name of the tab
    fn tab_name(&self) -> TabNameHolder {
        CustomTabName::TraitBased.into()
    }
}

/// This function is a system that will be called every frame and will construct tab ui
fn system_tab(mut commands: Commands, mut ui: NonSendMut<EditorUiRef>) {
    let ui = &mut ui.0;

    ui.label("System Based");

    // If button is clicked spawn a new entity with
    // SpatialBundle and Name components
    if ui.button("Add").clicked() {
        commands.spawn((
            SpatialBundle::default(),
            PrefabMarker,
            Name::new("New Entity".to_string()),
        ));
    }
}
