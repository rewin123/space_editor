/// This library contains dock-tab implementation for Space Editor
pub mod editor_tab;
pub mod schedule_editor_tab;
pub mod start_layout;
pub mod tab_name;
pub mod tab_style;
pub mod tab_viewer;

use std::fmt::Display;

use bevy::{ecs::world::CommandQueue, prelude::*, utils::HashMap, window::PrimaryWindow};

use bevy_egui::{egui, EguiContext};

use editor_tab::*;
use egui_dock::DockArea;
use schedule_editor_tab::*;
use start_layout::StartLayout;
use tab_name::{TabName, TabNameHolder};
use tab_style::*;
use tab_viewer::*;

pub mod prelude {
    pub use super::editor_tab::*;
    pub use super::schedule_editor_tab::*;
    pub use super::start_layout::*;
    pub use super::tab_name::*;
    pub use super::tab_style::*;
    pub use super::tab_viewer::*;

    pub use super::{
        show_editor_ui, EditorTabGetTitleFn, EditorTabShowFn, EditorUi, EditorUiAppExt,
        EditorUiReg, NewTabBehaviour, NewWindowSettings,
    };
}

/// This system use to show all egui editor ui on primary window
/// Will be useful in some specific cases to ad new system before/after this system
pub fn show_editor_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();
    egui_extras::install_image_loaders(ctx);

    // set style for editor
    if let Some(editor_ui) = world.get_resource::<EditorUi>() {
        let tab_style = (editor_ui.style_getter)(world);
        ctx.style_mut(|stl| {
            tab_style.set_egui_style(world, stl);
        });

        world.resource_scope::<EditorUi, _>(|world, mut editor_ui| {
            editor_ui.ui(world, ctx);
        });
    }
}

/// This resource contains registered editor tabs and current dock tree state
#[derive(Resource)]
pub struct EditorUi {
    pub registry: HashMap<TabNameHolder, EditorUiReg>,
    pub tree: egui_dock::DockState<TabNameHolder>,
    pub style_getter: fn(&World) -> &dyn TabStyle,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            registry: HashMap::default(),
            tree: egui_dock::DockState::new(vec![]),
            style_getter: |_| &DEFAULT_STYLE,
        }
    }
}

impl EditorUi {
    pub fn set_style<T: TabStyle>(&mut self) {
        self.style_getter = |world: &World| world.resource::<T>();
    }

    pub fn set_layout<T: StartLayout>(&mut self, layout: &T) {
        self.tree = layout.build();
    }

    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        //collect tab names to vec to detect visible
        let mut visible = vec![];
        for (_surface_index, tab) in self.tree.iter_all_nodes() {
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

        let collected_style = {
            let editor_style = (self.style_getter)(world);
            editor_style.collect_style(world)
        };

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
                style: collected_style,
            }
        };

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(ctx, &mut tab_viewer);

        let Some(windows_setting) =
            (unsafe { cell.world_mut().get_resource_mut::<NewWindowSettings>() })
        else {
            error!("Failed to load new window settings");
            return;
        };
        for command in tab_viewer.tab_commands {
            match command {
                EditorTabCommand::Add {
                    name,
                    surface,
                    node,
                } => match windows_setting.new_tab {
                    NewTabBehaviour::Pop => {
                        self.tree.add_window(vec![name]);
                    }
                    NewTabBehaviour::SameNode => {
                        if let Some(tree) = self
                            .tree
                            .get_surface_mut(surface)
                            .and_then(|surface| surface.node_tree_mut())
                        {
                            tree.set_focused_node(node);
                            tree.push_to_focused_leaf(name);
                        }
                    }
                    NewTabBehaviour::SplitNode => {
                        if let Some(surface) = self.tree.get_surface_mut(surface) {
                            surface
                                .node_tree_mut()
                                .unwrap() // Guaranteed to exist
                                .split_right(node, 0.5, vec![name]);
                        }
                    }
                },
            }
        }

        unsafe {
            command_queue.apply(cell.world_mut());
        }
    }
}

/// Trait for registering editor tabs via app.**
pub trait EditorUiAppExt {
    fn editor_tab_by_trait<T>(&mut self, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static;
    fn editor_tab<T, N: TabName>(
        &mut self,
        tab_name: N,
        tab_systems: impl IntoSystemConfigs<T>,
    ) -> &mut Self;
}

impl EditorUiAppExt for App {
    fn editor_tab_by_trait<T>(&mut self, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static,
    {
        let tab_name = tab.tab_name();

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
            title_command: Box::new(|world| {
                let text_size =
                    world
                        .get_resource::<EditorUi>()
                        .map_or(DEFAULT_TAB_FONT_SIZE, |editor| {
                            let editor_style = (editor.style_getter)(world);
                            editor_style.text_size(world)
                        });

                to_label(
                    &world
                        .get_resource::<T>()
                        .map(|tab| tab.tab_name().title)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    text_size,
                )
                .into()
            }),
        };

        if let Some(mut editor) = self.world_mut().get_resource_mut::<EditorUi>() {
            editor.registry.insert(tab_name, reg);
        };
        self
    }

    fn editor_tab<T, N: TabName>(
        &mut self,
        tab_name: N,
        tab_systems: impl IntoSystemConfigs<T>,
    ) -> &mut Self {
        let tab_name_holder = TabNameHolder::new(tab_name);

        let mut tab = ScheduleEditorTab {
            schedule: Schedule::default(),
            tab_name: tab_name_holder.clone(),
        };

        tab.schedule.add_systems(tab_systems);

        // Not much we can do here
        self.world_mut()
            .resource_mut::<ScheduleEditorTabStorage>()
            .0
            .insert(tab_name_holder.clone(), tab);
        if let Some(mut editor) = self.world_mut().get_resource_mut::<EditorUi>() {
            editor
                .registry
                .insert(tab_name_holder, EditorUiReg::Schedule);
        };
        self
    }
}

pub type EditorTabShowFn = Box<dyn Fn(&mut egui::Ui, &mut Commands, &mut World) + Send + Sync>;
pub type EditorTabGetTitleFn = Box<dyn Fn(&mut World) -> egui::WidgetText + Send + Sync>;

/// This enum determine how tab was registered.
/// ResourceBased - tab will be registered as resource
/// Schedule - tab will be registered as system
pub enum EditorUiReg {
    ResourceBased {
        show_command: EditorTabShowFn,
        title_command: EditorTabGetTitleFn,
    },
    Schedule,
}

#[derive(Default, Reflect, PartialEq, Eq, Clone)]
pub enum NewTabBehaviour {
    Pop,
    #[default]
    SameNode,
    SplitNode,
}

impl Display for NewTabBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pop => write!(f, "New window"),
            Self::SameNode => write!(f, "Same Node"),
            Self::SplitNode => write!(f, "Splits Node"),
        }
    }
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct NewWindowSettings {
    pub new_tab: NewTabBehaviour,
}

impl NewWindowSettings {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::new("new_tab", "")
            .selected_text(self.new_tab.to_string())
            .show_ui(ui, |ui| {
                for mode in TAB_MODES.into_iter() {
                    if ui
                        .selectable_label(self.new_tab == mode, mode.to_string())
                        .clicked()
                    {
                        self.new_tab = mode;
                    }
                }
            });
    }
}
