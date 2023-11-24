//This module contains ui logics, which will be work through events with editor core module and prefab module
mod mouse_check;

pub mod asset_inspector;
pub use asset_inspector::*;

pub mod bot_menu;
pub use bot_menu::*;

pub mod hierarchy;
use egui_dock::DockArea;
pub use hierarchy::*;

pub mod inspector;
pub use inspector::*;

pub mod editor_tab;
pub use editor_tab::*;

pub mod game_view;
pub use game_view::*;

pub mod settings;
pub use settings::*;

pub mod tools;
pub use tools::*;

pub mod change_chain;
pub use change_chain::*;

pub mod debug_panels;

use bevy::{ecs::system::CommandQueue, prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext};

use crate::{EditorSet, EditorState};

use self::{
    mouse_check::{pointer_context_check, MouseCheck},
    tools::gizmo::GizmoTool,
};

use super::{
    core::{SelectedPlugin, ToolExt, UndoRedo},
    update_pan_orbit,
};

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct UiSystemSet;

pub struct EditorUiPlugin {
    pub use_standard_layout: bool,
}

impl Default for EditorUiPlugin {
    fn default() -> Self {
        Self {
            use_standard_layout: true,
        }
    }
}

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.add_plugins((bot_menu::BotMenuPlugin, MouseCheck));

        app.configure_sets(
            Update,
            UiSystemSet
                .in_set(EditorSet::Editor)
                .run_if(in_state(EditorState::Editor)),
        );

        app.init_resource::<EditorUi>();
        app.init_resource::<ScheduleEditorTabStorage>();
        app.add_systems(
            Update,
            (
                show_editor_ui
                    .before(update_pan_orbit)
                    .before(super::ui_camera_block)
                    .after(bot_menu::bot_menu),
                set_camera_viewport.run_if(pointer_context_check()),
            )
                .in_set(UiSystemSet),
        );
        app.add_systems(
            Update,
            reset_camera_viewport.run_if(in_state(EditorState::Game)),
        );
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());

        app.editor_tab_by_trait(
            EditorTabName::Other("Debug World Inspector".to_string()),
            self::debug_panels::DebugWorldInspector {},
        );

        app.add_plugins(SpaceHierarchyPlugin::default());
        app.add_plugins(SpaceInspectorPlugin);

        app.editor_tool(GizmoTool::default());
        app.world.resource_mut::<GameViewTab>().active_tool = Some(0);

        app.add_plugins(settings::SettingsWindowPlugin);
        app.add_plugins(ChangeChainViewPlugin);

        if self.use_standard_layout {
            let mut editor = app.world.resource_mut::<EditorUi>();
            editor.tree = egui_dock::DockState::new(vec![EditorTabName::GameView]);

            let [_game, _inspector] = editor.tree.main_surface_mut().split_right(
                egui_dock::NodeIndex::root(),
                0.8,
                vec![EditorTabName::Inspector],
            );
            let [_hierarchy, _game] = editor.tree.main_surface_mut().split_left(
                _game,
                0.2,
                vec![EditorTabName::Hierarchy],
            );
        }
    }
}

fn show_editor_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<EditorUi, _>(|world, mut editor_ui| {
        editor_ui.ui(world, egui_context.get_mut());
    });
}

#[derive(Resource)]
pub struct EditorUi {
    pub registry: HashMap<EditorTabName, EditorUiReg>,
    pub tree: egui_dock::DockState<EditorTabName>,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            registry: HashMap::default(),
            tree: egui_dock::DockState::new(vec![]),
        }
    }
}

pub enum EditorUiReg {
    ResourceBased {
        show_command: EditorTabShowFn,
        title_command: EditorTabGetTitleFn,
    },
    Schedule,
}

impl EditorUi {
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        //collect tab names to vec to detect visible
        let mut visible = vec![];
        for tab in self.tree.iter_nodes() {
            match tab {
                egui_dock::Node::Empty => {}
                egui_dock::Node::Leaf {
                    rect: _,
                    viewport: _,
                    tabs,
                    active: _,
                    scroll: _,
                } => visible.extend(tabs.clone()),
                egui_dock::Node::Vertical {
                    rect: _,
                    fraction: _,
                } => {}
                egui_dock::Node::Horizontal {
                    rect: _,
                    fraction: _,
                } => {}
            }
        }

        let cell = world.as_unsafe_world_cell();

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, unsafe { cell.world() });

        let mut tab_viewer = unsafe {
            EditorTabViewer {
                commands: &mut commands,
                world: cell.world_mut(),
                registry: &mut self.registry,
                visible,
                tab_commands: vec![],
            }
        };

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(ctx, &mut tab_viewer);

        for command in tab_viewer.tab_commands {
            match command {
                EditorTabCommand::Add {
                    name,
                    surface,
                    node,
                } => {
                    if let Some(surface) = self.tree.get_surface_mut(surface) {
                        surface
                            .node_tree_mut()
                            .unwrap()
                            .split_right(node, 0.5, vec![name]);
                    }
                }
            }
        }

        unsafe {
            command_queue.apply(cell.world_mut());
        }
    }
}

pub trait EditorUiAppExt {
    fn editor_tab_by_trait<T>(&mut self, tab_id: EditorTabName, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static;
    fn editor_tab<T>(
        &mut self,
        tab_id: EditorTabName,
        title: egui::WidgetText,
        tab_systesm: impl IntoSystemConfigs<T>,
    ) -> &mut Self;
}

impl EditorUiAppExt for App {
    fn editor_tab_by_trait<T>(&mut self, tab_id: EditorTabName, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static,
    {
        self.insert_resource(tab);
        let show_fn = Box::new(
            |ui: &mut egui::Ui, commands: &mut Commands, world: &mut World| {
                world.resource_scope(|scoped_world, mut data: Mut<T>| {
                    data.ui(ui, commands, scoped_world)
                });
            },
        );
        let reg = EditorUiReg::ResourceBased {
            show_command: show_fn,
            title_command: Box::new(|world| world.resource_mut::<T>().title()),
        };

        self.world
            .resource_mut::<EditorUi>()
            .registry
            .insert(tab_id, reg);
        self
    }

    fn editor_tab<T>(
        &mut self,
        tab_id: EditorTabName,
        title: egui::WidgetText,
        tab_systesm: impl IntoSystemConfigs<T>,
    ) -> &mut Self {
        let mut tab = ScheduleEditorTab {
            schedule: Schedule::default(),
            title,
        };

        tab.schedule.add_systems(tab_systesm);

        self.world
            .resource_mut::<ScheduleEditorTabStorage>()
            .0
            .insert(tab_id.clone(), tab);
        self.world
            .resource_mut::<EditorUi>()
            .registry
            .insert(tab_id, EditorUiReg::Schedule);
        self
    }
}

/// Temporary resource for pretty system, based tab registration
pub struct EditorUiRef(pub egui::Ui);
