#![allow(clippy::type_complexity)]

//This module contains ui logics, which will be work through events with editor core module and prefab module
mod mouse_check;

/// This module will be used to create Unity like project file dialog. Currently NOT USED
pub mod asset_inspector;

/// This module contains logic for menu toolbars
pub mod menu_toolbars;

/// This module contains UI logic for undo/redo functionality
pub mod change_chain;

/// This module contains UI logic for debug panels (like WorldInspector)
pub mod debug_panels;

/// This module contains traits and logic for editor dock tabs. Also it contains logic to run all editor dock ui
pub mod editor_tab;

/// This module contains Game view tab logic
pub mod game_view;

/// This module contains Hierarchy tab logic
pub mod hierarchy;

/// This module contains Inspector tab logic
pub mod inspector;

/// This module contains methods to visualize entities without a mesh attached
pub mod meshless_visualizer;

/// This module contains Settings tab logic
pub mod settings;

/// This module contains traits and methods to register tools in game view tab
pub mod tool;

/// This module contains IMPLEMENTATIONS for existed tools (like Gizmo manipulation tool)
pub mod tools;

/// This module contains methods for bundle registration
pub mod ui_registration;

/// This module contains UI logic for view game camera image
pub mod camera_view;


/// UI plugin and common systems
pub mod ui_plugin;

/// Camera plugin and logic
pub mod camera_plugin;

///Selection logic
pub mod selection;

pub mod icons;

use bevy_debug_grid::{Grid, GridAxis, SubGrid, TrackedGrid, DEFAULT_GRID_ALPHA};
use bevy_mod_picking::{
    backends::raycast::RaycastPickable,
    events::{Down, Pointer},
    picking_core::Pickable,
    pointer::PointerButton,
    prelude::*,
    PickableBundle,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};
use camera_view::CameraViewTabPlugin;
use egui_dock::DockArea;
use selection::{delete_selected, SelectEvent};
use space_editor_core::prelude::*;

use bevy::{
    app::PluginGroupBuilder,
    ecs::system::CommandQueue,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{render_resource::PrimitiveTopology, view::RenderLayers},
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_egui_next::{egui, EguiContext};

use game_view::{has_window_changed, GameViewPlugin};
use prelude::{
    clean_meshless, reset_camera_viewport, set_camera_viewport, ChangeChainViewPlugin, EditorTab,
    EditorTabCommand, EditorTabGetTitleFn, EditorTabName, EditorTabShowFn, EditorTabViewer,
    GameModeSettings, GameViewTab, MeshlessVisualizerPlugin, NewTabBehaviour, NewWindowSettings,
    ScheduleEditorTab, ScheduleEditorTabStorage, SpaceHierarchyPlugin, SpaceInspectorPlugin,
};
use space_prefab::prelude::*;
use space_shared::{
    ext::bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin},
    EditorCameraMarker, EditorSet, EditorState, PrefabMarker, PrefabMemoryCache, SelectParent,
};
use space_undo::{SyncUndoMarkersPlugin, UndoPlugin, UndoSet};
use ui_registration::BundleReg;

use camera_plugin::*;
use ui_plugin::*;

use self::{mouse_check::MouseCheck, tools::gizmo::GizmoToolPlugin};

pub const LAST_RENDER_LAYER: u8 = RenderLayers::TOTAL_LAYERS as u8 - 1;

pub mod prelude {
    pub use super::{
        asset_inspector::*, change_chain::*, debug_panels::*, editor_tab::*, game_view::*,
        hierarchy::*, inspector::*, menu_toolbars::*, meshless_visualizer::*, settings::*, tool::*,
        tools::*, ui_registration::*,
    };

    pub use space_editor_core::prelude::*;
    pub use space_persistence::*;
    pub use space_prefab::prelude::*;
    pub use space_shared::prelude::*;

    pub use crate::camera_plugin::*;
    pub use crate::selection::*;
    pub use crate::simple_editor_setup;
    pub use crate::ui_plugin::*;
    pub use crate::EditorPlugin;
}

/// External dependencies for editor crate
pub mod ext {
    pub use bevy_egui_next;
    pub use bevy_mod_picking;
    pub use bevy_panorbit_camera;
    pub use space_shared::ext::*;
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorPluginGroup);
    }
}

/// Editor UI plugin. Must be used with [`PrefabPlugin`] and [`EditorRegistryPlugin`]
///
/// [`PrefabPlugin`]: prefab::prefabPlugin
/// [`EditorRegistryPlugin`]: crate::editor_registry::EditorRegistryPlugin
pub struct EditorPluginGroup;

impl PluginGroup for EditorPluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut res = PluginGroupBuilder::start::<Self>()
            .add(UndoPlugin)
            .add(SyncUndoMarkersPlugin::<PrefabMarker>::default())
            .add(PrefabPlugin)
            .add(space_editor_core::EditorCore)
            .add(EditorSetsPlugin)
            .add(EditorDefaultBundlesPlugin)
            .add(EditorDefaultCameraPlugin)
            .add(bevy_egui_next::EguiPlugin)
            .add(EventListenerPlugin::<selection::SelectEvent>::default())
            .add(DefaultInspectorConfigPlugin);
        res = EditorUiPlugin::default().add_plugins_to_group(res);
        res.add(PanOrbitCameraPlugin)
            .add(selection::EditorPickingPlugin)
            .add(bevy_debug_grid::DebugGridPlugin::without_floor_grid())
            .add(
                WorldInspectorPlugin::default()
                    .run_if(in_state(EditorState::Game))
                    .run_if(input_toggle_active(false, KeyCode::Escape)),
            )
            .add(EditorGizmoConfigPlugin)
    }
}

pub struct EditorDefaultBundlesPlugin;

impl Plugin for EditorDefaultBundlesPlugin {
    fn build(&self, app: &mut App) {
        ui_registration::register_mesh_editor_bundles(app);
        ui_registration::register_light_editor_bundles(app);
    }
}

pub struct EditorSetsPlugin;

impl Plugin for EditorSetsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PostUpdate, UndoSet::Global.in_set(EditorSet::Editor));

        app.configure_sets(
            PreUpdate,
            EditorSet::Game.run_if(in_state(EditorState::Game)),
        );
        app.configure_sets(Update, EditorSet::Game.run_if(in_state(EditorState::Game)));
        app.configure_sets(
            PostUpdate,
            EditorSet::Game.run_if(in_state(EditorState::Game)),
        );

        app.configure_sets(
            PreUpdate,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );
        app.configure_sets(
            Update,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );
        app.configure_sets(
            PostUpdate,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );

        app.configure_sets(Update, EditorSet::Game.run_if(in_state(EditorState::Game)));
        app.configure_sets(
            Update,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );
    }
}

/// Allow editor manipulate GizmoConfig
pub struct EditorGizmoConfigPlugin;

impl Plugin for EditorGizmoConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, editor_gizmos);
        app.add_systems(Update, game_gizmos);
    }
}

fn editor_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.render_layers = RenderLayers::layer(LAST_RENDER_LAYER)
}

fn game_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.render_layers = RenderLayers::layer(0)
}

type AutoAddQueryFilter = (
    Without<PrefabMarker>,
    Without<Pickable>,
    With<Parent>,
    Changed<Handle<Mesh>>,
);

fn save_prefab_before_play(mut editor_events: EventWriter<space_shared::EditorEvent>) {
    editor_events.send(space_shared::EditorEvent::Save(
        space_shared::EditorPrefabPath::MemoryCahce,
    ));
}

fn to_game_after_save(mut state: ResMut<NextState<EditorState>>) {
    info!("Set game state");
    state.set(EditorState::Game);
}

fn set_start_state(mut state: ResMut<NextState<EditorState>>) {
    info!("Set start state");
    state.set(EditorState::Loading);
}

fn clear_and_load_on_start(
    mut load_server: ResMut<EditorLoader>,
    save_confg: Res<SaveConfig>,
    assets: Res<AssetServer>,
    cache: Res<PrefabMemoryCache>,
) {
    if save_confg.path.is_none() {
        return;
    }
    match save_confg.path.as_ref().unwrap() {
        space_shared::EditorPrefabPath::File(path) => {
            info!("Loading prefab from file {}", path);
            load_server.scene = Some(assets.load(format!("{}.scn.ron", path)));
        }
        space_shared::EditorPrefabPath::MemoryCahce => {
            info!("Loading prefab from cache");
            load_server.scene = cache.scene.clone();
        }
    }
}

pub trait FlatPluginList {
    fn add_plugins_to_group(&self, group: PluginGroupBuilder) -> PluginGroupBuilder;
}

///Camera with this component will not be disabled in Editor state
#[derive(Component)]
pub struct DisableCameraSkip;

fn disable_no_editor_cams(
    mut cameras: Query<&mut Camera, (Without<DisableCameraSkip>, Without<EditorCameraMarker>)>,
) {
    for mut cam in cameras.iter_mut() {
        cam.is_active = false;
    }
}

#[derive(Component)]
pub struct NotShowCamera;

fn draw_camera_gizmo(
    mut gizmos: Gizmos,
    cameras: Query<
        (&GlobalTransform, &Projection),
        (
            With<Camera>,
            Without<EditorCameraMarker>,
            Without<DisableCameraSkip>,
            Without<NotShowCamera>,
        ),
    >,
) {
    for (transform, _projection) in cameras.iter() {
        let transform = transform.compute_transform();
        let cuboid_transform = transform.with_scale(Vec3::new(1.0, 1.0, 2.0));
        gizmos.cuboid(cuboid_transform, Color::PINK);

        let scale = 1.5;

        gizmos.line(
            transform.translation,
            transform.translation
                + transform.forward() * scale
                + transform.up() * scale
                + transform.right() * scale,
            Color::PINK,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale - transform.up() * scale
                + transform.right() * scale,
            Color::PINK,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale + transform.up() * scale
                - transform.right() * scale,
            Color::PINK,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale
                - transform.up() * scale
                - transform.right() * scale,
            Color::PINK,
        );

        let rect_transform = Transform::from_xyz(0.0, 0.0, -scale);
        let rect_transform = transform.mul_transform(rect_transform);

        gizmos.rect(
            rect_transform.translation,
            rect_transform.rotation,
            Vec2::splat(scale * 2.0),
            Color::PINK,
        );
    }
}

/// All systems for editor ui wil be placed in UiSystemSet
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct UiSystemSet;

/// Plugin for editor ui
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

/// State to determine if editor ui should be shown (ot hidden for any reason)
#[derive(Hash, PartialEq, Eq, Debug, Clone, States, Default)]
pub enum ShowEditorUi {
    #[default]
    Show,
    Hide,
}

impl FlatPluginList for EditorUiPlugin {
    fn add_plugins_to_group(&self, group: PluginGroupBuilder) -> PluginGroupBuilder {
        let mut res = group
            .add(SelectedPlugin::default())
            .add(EditorUiCore::default())
            .add(GameViewPlugin)
            .add(menu_toolbars::BottomMenuPlugin)
            .add(MouseCheck)
            .add(CameraViewTabPlugin)
            .add(SpaceHierarchyPlugin::default())
            .add(SpaceInspectorPlugin)
            .add(GizmoToolPlugin)
            .add(ChangeChainViewPlugin)
            .add(settings::SettingsWindowPlugin);

        if self.use_standard_layout {
            res = res.add(DefaultEditorLayoutPlugin);
        }

        res
    }
}

impl PluginGroup for EditorUiPlugin {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = self.add_plugins_to_group(group);
        group
    }
}

pub struct DefaultEditorLayoutPlugin;

impl Plugin for DefaultEditorLayoutPlugin {
    fn build(&self, app: &mut App) {
        let mut editor = app.world.resource_mut::<EditorUi>();
        editor.tree = egui_dock::DockState::new(vec![EditorTabName::GameView]);

        let [_game, _inspector] = editor.tree.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.8,
            vec![EditorTabName::Inspector],
        );
        let [_hierarchy, _game] =
            editor
                .tree
                .main_surface_mut()
                .split_left(_game, 0.2, vec![EditorTabName::Hierarchy]);
    }
}

pub struct EditorUiCore {
    pub disable_no_editor_cams: bool,
}

impl Default for EditorUiCore {
    fn default() -> Self {
        Self {
            disable_no_editor_cams: true,
        }
    }
}

impl Plugin for EditorUiCore {
    fn build(&self, app: &mut App) {
        app.add_state::<ShowEditorUi>();

        app.configure_sets(
            Update,
            UiSystemSet
                .in_set(EditorSet::Editor)
                .run_if(in_state(ShowEditorUi::Show)),
        );
        app.init_resource::<EditorUi>();
        app.init_resource::<ScheduleEditorTabStorage>();
        app.add_systems(
            Update,
            (
                show_editor_ui
                    .before(update_pan_orbit)
                    .before(ui_camera_block),
                set_camera_viewport,
            )
                .in_set(UiSystemSet),
        );

        app.add_systems(
            PostUpdate,
            set_camera_viewport
                .run_if(has_window_changed)
                .in_set(UiSystemSet),
        );
        app.add_systems(
            Update,
            reset_camera_viewport.run_if(in_state(EditorState::Game)),
        );
        app.add_systems(OnEnter(ShowEditorUi::Hide), reset_camera_viewport);
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());

        app.editor_tab_by_trait(
            EditorTabName::Other("Debug World Inspector".to_string()),
            self::debug_panels::DebugWorldInspector {},
        );

        app.init_resource::<EditorLoader>();

        app.insert_resource(EditorCameraEnabled(true));

        app.add_systems(
            Startup,
            (set_start_state, apply_state_transition::<EditorState>).chain(),
        );

        //play systems
        app.add_systems(OnEnter(EditorState::GamePrepare), save_prefab_before_play);
        app.add_systems(
            OnEnter(SaveState::Idle),
            to_game_after_save.run_if(in_state(EditorState::GamePrepare)),
        );

        app.add_systems(OnEnter(EditorState::Game), change_camera_in_play);

        app.add_systems(
            OnEnter(EditorState::Editor),
            (clear_and_load_on_start, set_camera_viewport),
        );

        app.add_systems(
            Update,
            (draw_camera_gizmo, delete_selected)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );

        if self.disable_no_editor_cams {
            app.add_systems(
                Update,
                disable_no_editor_cams.run_if(in_state(EditorState::Editor)),
            );

            app.add_systems(OnEnter(EditorState::Editor), change_camera_in_editor);
        }

        app.add_event::<SelectEvent>();

        app.init_resource::<BundleReg>();
    }
}

/// This system use to show all egui editor ui on primary window
/// Will be usefull in some specific cases to ad new system before/after this system
pub fn show_editor_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    info!("Show editor ui");

    world.resource_scope::<EditorUi, _>(|world, mut editor_ui| {
        editor_ui.ui(world, egui_context.get_mut());
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

/// This method prepare default lights and camera for editor UI. You can create own conditions for your editor and use this method how example
pub fn simple_editor_setup(mut commands: Commands, mut show_ui: ResMut<NextState<ShowEditorUi>>, mut editor_state: ResMut<NextState<EditorState>>) {
    show_ui.set(ShowEditorUi::Hide);
    editor_state.set(EditorState::Editor);

    commands.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 });
    // light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            cascade_shadow_config: CascadeShadowConfigBuilder::default().into(),
            ..default()
        },
        Name::from("Editor Level Light"),
    ));

    // grid
    let grid_render_layer = RenderLayers::layer(LAST_RENDER_LAYER);
    commands.spawn((
        Grid {
            spacing: 10.0_f32,
            count: 16,
            color: Color::SILVER.with_a(DEFAULT_GRID_ALPHA),
            alpha_mode: AlphaMode::Blend,
        },
        SubGrid {
            count: 9,
            color: Color::GRAY.with_a(DEFAULT_GRID_ALPHA),
        },
        GridAxis::new_rgb(),
        TrackedGrid::default(),
        TransformBundle::default(),
        VisibilityBundle::default(),
        Name::from("Debug Grid"),
        grid_render_layer,
    ));

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        bevy_panorbit_camera::PanOrbitCamera::default(),
        EditorCameraMarker,
        Name::from("Editor Camera"),
        PickableBundle::default(),
        RaycastPickable,
        RenderLayers::all(),
    ));
}

pub fn game_mode_changed(
    mut commands: Commands,
    mode: Res<GameModeSettings>,
    editor_camera_query: Query<Entity, (With<EditorCameraMarker>, With<Camera>)>,
) {
    if mode.is_changed() {
        for editor_camera in editor_camera_query.iter() {
            commands.entity(editor_camera).despawn_recursive();
        }

        if mode.is_3d() {
            // 3D camera
            commands.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                    camera: Camera {
                        order: 0,
                        ..default()
                    },
                    ..default()
                },
                bevy_panorbit_camera::PanOrbitCamera::default(),
                EditorCameraMarker,
                Name::from("Editor Camera"),
                PickableBundle::default(),
                RaycastPickable,
                RenderLayers::all(),
            ));
        } else {
            // 2D camera
            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: 0,
                        ..default()
                    },
                    ..default()
                },
                EditorCameraMarker,
                Name::from("Editor 2D Camera"),
                PickableBundle::default(),
                RaycastPickable,
                RenderLayers::all(),
            ));
        }
    }
}

pub mod colors {
    use bevy_egui_next::egui::{Color32, Stroke};

    pub fn stroke_default_color() -> Stroke {
        Stroke::new(1., STROKE_COLOR)
    }
    pub const STROKE_COLOR: Color32 = Color32::from_rgb(70, 70, 70);
    pub const SPECIAL_BG_COLOR: Color32 = Color32::from_rgb(20, 20, 20);
    pub const DEFAULT_BG_COLOR: Color32 = Color32::from_rgb(27, 27, 27);
    pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 59, 33);
    pub const HYPERLINK_COLOR: Color32 = Color32::from_rgb(99, 235, 231);
    pub const WARM_COLOR: Color32 = Color32::from_rgb(225, 206, 67);
    pub const SELECTED_ITEM_COLOR: Color32 = Color32::from_rgb(76, 93, 235);
    pub const TEXT_COLOR: Color32 = Color32::WHITE;
}
