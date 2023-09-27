//This module contains ui logics, which will be work through events with editor core module and prefab module


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

use bevy_egui::{egui, EguiContext};
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::{prelude::SelectedPlugin, EditorSet};

use super::update_pan_orbit;


pub struct EditorUiPlugin {
    pub use_standart_layout : bool
}

impl Default for EditorUiPlugin {
    fn default() -> Self {
        Self {
            use_standart_layout : true
        }
    }
}

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.add_plugins(bot_menu::BotMenuPlugin);

        app.init_resource::<EditorUi>();
        app.init_resource::<ScheduleEditorTabStorage>();
        app.add_systems(Update, (
            show_editor_ui.before(update_pan_orbit).after(bot_menu::bot_menu),
            set_camera_viewport
        ).in_set(EditorSet::Editor));
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());


        app.add_plugins(SpaceHierarchyPlugin::default());
        app.add_plugins(SpaceInspectorPlugin);

        if self.use_standart_layout {
            let mut editor = app.world.resource_mut::<EditorUi>();
            editor.tree = egui_dock::DockState::new(vec![
                EditorTabName::GameView
            ]);

            let [_game, right_panel] = editor.tree.main_surface_mut().split_right(egui_dock::NodeIndex::root(), 0.75, vec![EditorTabName::Hierarchy]);
            let [_hierarchy, _inspector] = editor.tree.main_surface_mut().split_below(right_panel, 0.5, vec![EditorTabName::Inspector]);
        }
    }
}

fn show_editor_ui(
    world : &mut World
) {
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
    pub registry : HashMap<EditorTabName, EditorUiReg>,
    pub tree : egui_dock::DockState<EditorTabName>
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            registry : HashMap::default(),
            tree : egui_dock::DockState::new(vec![])
        }
    }
}

pub enum EditorUiReg {
    ResourceBased{show_command : EditorTabShowFn,
        title_command : EditorTabGetTitleFn},
    Schedule
}

impl EditorUi {
    pub fn ui(&mut self, world : &mut World, ctx : &mut egui::Context) {
        //collect tab names to vec to detect visible
        let mut visible = vec![];
        for tab in self.tree.iter_nodes() {
            match tab {
                egui_dock::Node::Empty => {},
                egui_dock::Node::Leaf { rect: _, viewport: _, tabs, active: _, scroll: _ } => visible.extend(tabs.clone()),
                egui_dock::Node::Vertical { rect: _, fraction: _ } => {},
                egui_dock::Node::Horizontal { rect: _, fraction: _ } => {},
            }
        }

        
        let mut tab_viewer = EditorTabViewer {
            world,
            registry : &mut self.registry,
            visible,
            commands: vec![],
        };

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(ctx, &mut tab_viewer);

        for command in tab_viewer.commands {
            match command {
                EditorTabCommand::Add { name, surface, node } => {
                    if let Some(surface) = self.tree.get_surface_mut(surface) {
                        surface.node_tree_mut().unwrap().split_right(node, 0.5, vec![name]);
                    }
                },
            }
        }
    }
}

pub trait EditorUiAppExt {
    fn editor_tab_by_trait<T>(&mut self, tab_id : EditorTabName, tab : T) -> &mut Self where T : EditorTab + Resource + Send + Sync + 'static;
    fn editor_tab<T>(&mut self, tab_id : EditorTabName, title : egui::WidgetText, tab_systesm : impl IntoSystemConfigs<T>) -> &mut Self;
}

impl EditorUiAppExt for App {
    fn editor_tab_by_trait<T>(&mut self, tab_id : EditorTabName, tab : T) -> &mut Self where T : EditorTab + Resource + Send + Sync + 'static {
        self.insert_resource(tab);
        let show_fn = Box::new(|ui : &mut egui::Ui, world : &mut World| {
            world.resource_scope(|scoped_world, mut data : Mut<T>| {
                data.ui(ui, scoped_world)
            });
        });
        let reg = EditorUiReg::ResourceBased { 
            show_command: show_fn, 
            title_command: Box::new(|world| {
                world.resource_mut::<T>().title()
            }) 
        };

        self.world.resource_mut::<EditorUi>().registry.insert(tab_id, reg);
        self
    }

    fn editor_tab<T>(&mut self, tab_id : EditorTabName, title : egui::WidgetText, tab_systesm : impl IntoSystemConfigs<T>) -> &mut Self {
        let mut tab = ScheduleEditorTab {
            schedule: Schedule::new(),
            title,
        };

        tab.schedule.add_systems(tab_systesm);

        self.world.resource_mut::<ScheduleEditorTabStorage>().0.insert(tab_id.clone(), tab);
        self.world.resource_mut::<EditorUi>().registry.insert(tab_id, EditorUiReg::Schedule);
        self
    }
}


/// Temporary resource for pretty system, based tab registration
pub struct EditorUiRef(pub egui::Ui);