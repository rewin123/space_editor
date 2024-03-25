/// This library contains dock-tab implementation for Space Editor
pub mod editor_tab;
pub mod schedule_editor_tab;
pub mod colors;
pub mod sizing;
pub mod tab_viewer;

use bevy::{ecs::system::CommandQueue, prelude::*, utils::HashMap, window::PrimaryWindow};

use bevy_egui::{egui::{self, FontId, Rounding}, EguiContext};
use bevy_egui::egui::FontFamily::{Monospace, Proportional};

use editor_tab::*;
use colors::*;
use egui_dock::DockArea;
use sizing::{to_label, Sizing};

use bevy_egui::egui::TextStyle as ETextStyle;

pub mod prelude {
    pub use super::*;
    pub use editor_tab::*;
    pub use colors::*;
    pub use super::sizing::*;
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
    ctx.style_mut(|stl| {
        stl.spacing.button_padding = bevy_egui::egui::Vec2::new(8., 2.);
        stl.spacing.icon_spacing = 4.;
        stl.spacing.icon_width = 16.;
        stl.spacing.menu_margin = bevy_egui::egui::Margin {
            left: 8.,
            right: 8.,
            top: 4.,
            bottom: 8.,
        };
        stl.visuals.error_fg_color = ERROR_COLOR;
        stl.visuals.hyperlink_color = HYPERLINK_COLOR;
        stl.visuals.warn_fg_color = WARM_COLOR;
        stl.visuals.menu_rounding = Rounding::same(0.5);
        stl.text_styles = [
            (ETextStyle::Small, FontId::new(10.0, Proportional)),
            (ETextStyle::Body, FontId::new(12., Proportional)),
            (ETextStyle::Button, FontId::new(14., Proportional)),
            (ETextStyle::Heading, FontId::new(20.0, Proportional)),
            (ETextStyle::Monospace, FontId::new(12.0, Monospace)),
        ]
        .into()
    });

    world.resource_scope::<EditorUi, _>(|world, mut editor_ui| {
        editor_ui.ui(world, ctx);
    });
}

/// This resource contains registered editor tabs and current dock tree state
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

impl EditorUi {
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

        let windows_setting = unsafe { cell.world_mut().resource_mut::<NewWindowSettings>() };
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
                                .unwrap()
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
    fn editor_tab_by_trait<T>(&mut self, tab_id: EditorTabName, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static;
    fn editor_tab<T>(
        &mut self,
        tab_id: EditorTabName,
        title: egui::WidgetText,
        tab_systems: impl IntoSystemConfigs<T>,
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
            title_command: Box::new(|world| {
                let sizing = world.resource::<Sizing>().clone();
                to_label(world.resource_mut::<T>().title().text(), sizing.text).into()
            }),
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
        tab_systems: impl IntoSystemConfigs<T>,
    ) -> &mut Self {
        let mut tab = ScheduleEditorTab {
            schedule: Schedule::default(),
            title,
        };

        tab.schedule.add_systems(tab_systems);

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


#[derive(Default, Reflect, PartialEq, Eq, Clone)]
pub enum NewTabBehaviour {
    Pop,
    #[default]
    SameNode,
    SplitNode,
}

impl ToString for NewTabBehaviour {
    fn to_string(&self) -> String {
        match self {
            Self::Pop => "New window",
            Self::SameNode => "Same Node",
            Self::SplitNode => "Splits Node",
        }
        .to_string()
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
