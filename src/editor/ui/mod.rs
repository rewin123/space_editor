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
        app.init_resource::<EditorUi>();
        app.add_systems(Update, show_editor_ui);
        app.editor_tab(EditorTabName::GameView, GameViewTab::default());

        app.add_plugins(SpaceHierarchyPlugin::default());

        if self.use_standart_layout {
            let mut editor = app.world.resource_mut::<EditorUi>();
            editor.tree = egui_dock::Tree::new(vec![
                EditorTabName::GameView
            ]);

            let [game, right_panel] = editor.tree.split_right(egui_dock::NodeIndex::root(), 0.75, vec![EditorTabName::Hierarchy]);
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

#[derive(Resource, Default)]
pub struct EditorUi {
    pub show_commands : HashMap<EditorTabName, EditorTabShowFn>,
    pub title_commands : HashMap<EditorTabName, EditorTabGetTitleFn>,
    pub tree : egui_dock::Tree<EditorTabName>
}

impl EditorUi {
    pub fn ui(&mut self, world : &mut World, ctx : &mut egui::Context) {
        let mut tab_viewer = EditorTabViewer {
            world,
            title_commands: &mut self.title_commands,
            show_commands: &mut self.show_commands,
        };
        DockArea::new(&mut self.tree)
            .show(ctx, &mut tab_viewer);
    }
}

pub trait EditorUiAppExt {
    fn editor_tab<T>(&mut self, tab_id : EditorTabName, tab : T) where T : EditorTab + Resource + Send + Sync + 'static;
}

impl EditorUiAppExt for App {
    fn editor_tab<T>(&mut self, tab_id : EditorTabName, tab : T) where T : EditorTab + Resource + Send + Sync + 'static {
        self.insert_resource(tab);
        let show_fn = Box::new(|ui : &mut egui::Ui, world : &mut World| {
            world.resource_scope(|scoped_world, mut data : Mut<T>| {
                data.ui(ui, scoped_world)
            });
        });
        self.world.resource_mut::<EditorUi>().show_commands.insert(tab_id.clone(), show_fn);

        self.world.resource_mut::<EditorUi>().title_commands.insert(tab_id, Box::new(|world| {
            world.resource_mut::<T>().title()
        }));
    }
}
