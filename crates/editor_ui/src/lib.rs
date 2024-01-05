#![allow(clippy::type_complexity)]

//This module contains ui logics, which will be work through events with editor core module and prefab module
mod mouse_check;

/// This module will be used to create Unity like project file dialog. Currently NOT USED
pub mod asset_inspector;

/// This module contains logic for bottom menu
pub mod bottom_menu;

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
use space_editor_core::prelude::*;

use bevy::{
    ecs::system::CommandQueue,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{render_resource::PrimitiveTopology, view::RenderLayers},
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext};

use game_view::has_window_changed;
use prelude::{
    reset_camera_viewport, set_camera_viewport, ChangeChainViewPlugin, EditorTab, EditorTabCommand,
    EditorTabGetTitleFn, EditorTabName, EditorTabShowFn, EditorTabViewer, GameViewTab,
    MeshlessVisualizerPlugin, NewTabBehaviour, NewWindowSettings, ScheduleEditorTab,
    ScheduleEditorTabStorage, SpaceHierarchyPlugin, SpaceInspectorPlugin, ToolExt,
};
use space_prefab::prelude::*;
use space_shared::{
    ext::bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin},
    EditorCameraMarker, EditorSet, EditorState, PrefabMarker, PrefabMemoryCache, SelectParent,
};
use space_undo::UndoPlugin;
use ui_registration::BundleReg;

use self::{
    mouse_check::MouseCheck,
    tools::gizmo::{GizmoTool, GizmoToolPlugin},
};

pub mod prelude {
    pub use super::{
        asset_inspector::*, bottom_menu::*, change_chain::*, debug_panels::*, editor_tab::*,
        game_view::*, hierarchy::*, inspector::*, meshless_visualizer::*, settings::*, tool::*,
        tools::*, ui_registration::*,
    };

    pub use space_editor_core::prelude::*;
    pub use space_persistence::*;
    pub use space_prefab::prelude::*;
    pub use space_shared::prelude::*;

    pub use crate::simple_editor_setup;
    pub use crate::EditorPlugin;
    pub use crate::EditorUiAppExt;
    pub use crate::EditorUiRef;
}

/// External dependencies for editor crate
pub mod ext {
    pub use bevy_egui;
    pub use bevy_mod_picking;
    pub use bevy_panorbit_camera;
    pub use space_shared::ext::*;
}

/// Editor UI plugin. Must be used with [`PrefabPlugin`] and [`EditorRegistryPlugin`]
///
/// [`PrefabPlugin`]: prefab::prefabPlugin
/// [`EditorRegistryPlugin`]: crate::editor_registry::EditorRegistryPlugin
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }
        app.add_plugins(UndoPlugin); //Undo must be included before prefab plugin for undo registration
        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }
        app.add_plugins(EditorCore);

        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }

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

        app.add_plugins(EventListenerPlugin::<SelectEvent>::default());

        app.add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(EditorUiPlugin::default())
            .add_plugins(PanOrbitCameraPlugin);

        if !app.is_plugin_added::<bevy_mod_picking::prelude::SelectionPlugin>() {
            app.add_plugins(bevy_mod_picking::DefaultPickingPlugins);

            app.world
                .resource_mut::<bevy_mod_picking::backends::raycast::RaycastBackendSettings>()
                .require_markers = true;
        }

        if !app.is_plugin_added::<bevy_debug_grid::DebugGridPlugin>() {
            app.add_plugins(bevy_debug_grid::DebugGridPlugin::without_floor_grid());
        }
        app.init_resource::<EditorLoader>();

        app.insert_resource(PanOrbitEnabled(true));

        app.add_systems(
            Startup,
            (set_start_state, apply_state_transition::<EditorState>)
                .chain()
                .in_set(EditorSet::Editor),
        );

        app.add_systems(
            Update,
            reset_pan_orbit_state
                .in_set(EditorSet::Editor)
                .before(UiSystemSet),
        );
        app.add_systems(
            Update,
            update_pan_orbit
                .after(reset_pan_orbit_state)
                .before(PanOrbitCameraSystemSet)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(
            Update,
            ui_camera_block
                .after(reset_pan_orbit_state)
                .before(update_pan_orbit)
                .in_set(EditorSet::Editor),
        );

        app.add_systems(Update, draw_grid_lines.in_set(EditorSet::Editor));

        //play systems
        app.add_systems(
            OnEnter(EditorState::GamePrepare),
            (cleanup_grid_lines, save_prefab_before_play),
        );
        app.add_systems(
            OnEnter(SaveState::Idle),
            to_game_after_save.run_if(in_state(EditorState::GamePrepare)),
        );

        app.add_systems(OnEnter(EditorState::Game), change_camera_in_play);
        app.add_systems(OnEnter(EditorState::GamePrepare), game_gizmos);

        app.add_systems(
            OnEnter(EditorState::Editor),
            (
                clear_and_load_on_start,
                change_camera_in_editor,
                create_grid_lines,
                editor_gizmos,
                set_camera_viewport,
            ),
        );

        app.add_systems(
            PostUpdate,
            (auto_add_picking, select_listener.after(UiSystemSet))
                .run_if(in_state(EditorState::Editor)),
        );
        app.add_systems(PostUpdate, auto_add_picking_dummy);

        app.add_systems(
            Update,
            (draw_camera_gizmo, disable_no_editor_cams, delete_selected)
                .run_if(in_state(EditorState::Editor)),
        );

        app.add_event::<SelectEvent>();

        app.init_resource::<BundleReg>();

        app.add_plugins(
            WorldInspectorPlugin::default()
                .run_if(in_state(EditorState::Game))
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        );

        ui_registration::register_mesh_editor_bundles(app);
        ui_registration::register_light_editor_bundles(app);
    }
}

/// This event used for selecting entities
#[derive(Event, Clone, EntityEvent)]
struct SelectEvent {
    #[target]
    e: Entity,
    event: ListenerInput<Pointer<Down>>,
}

#[derive(Component)]
pub struct GridLines {
    pub cell_size: f32,
    pub half_cell_width: i32,
}

impl Default for GridLines {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            half_cell_width: 128,
        }
    }
}

fn editor_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.render_layers = RenderLayers::layer(RenderLayers::TOTAL_LAYERS as u8 - 1)
}

fn game_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.render_layers = RenderLayers::layer(0)
}

fn create_grid_lines(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), GridLines::default()));
}

fn cleanup_grid_lines(mut commands: Commands, query: Query<Entity, With<GridLines>>) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn draw_grid_lines(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &GridLines)>) {
    for (transform, grid) in query.iter() {
        let pos = transform.translation();
        for x in 1..grid.half_cell_width {
            gizmos.line(
                Vec3::new(
                    x as f32 * grid.cell_size,
                    0.0,
                    -grid.half_cell_width as f32 * grid.cell_size,
                ) + pos,
                Vec3::new(
                    x as f32 * grid.cell_size,
                    0.0,
                    grid.half_cell_width as f32 * grid.cell_size,
                ) + pos,
                Color::GRAY,
            );

            gizmos.line(
                Vec3::new(
                    -x as f32 * grid.cell_size,
                    0.0,
                    -grid.half_cell_width as f32 * grid.cell_size,
                ) + pos,
                Vec3::new(
                    -x as f32 * grid.cell_size,
                    0.0,
                    grid.half_cell_width as f32 * grid.cell_size,
                ) + pos,
                Color::GRAY,
            );
        }

        for z in 1..grid.half_cell_width {
            gizmos.line(
                Vec3::new(
                    -grid.half_cell_width as f32 * grid.cell_size,
                    0.0,
                    z as f32 * grid.cell_size,
                ) + pos,
                Vec3::new(
                    grid.half_cell_width as f32 * grid.cell_size,
                    0.0,
                    z as f32 * grid.cell_size,
                ) + pos,
                Color::GRAY,
            );

            gizmos.line(
                Vec3::new(
                    -grid.half_cell_width as f32 * grid.cell_size,
                    0.0,
                    -z as f32 * grid.cell_size,
                ) + pos,
                Vec3::new(
                    grid.half_cell_width as f32 * grid.cell_size,
                    0.0,
                    -z as f32 * grid.cell_size,
                ) + pos,
                Color::GRAY,
            );
        }

        //draw x central axis
        gizmos.line(
            Vec3::new(0.0, 0.0, -grid.half_cell_width as f32 * grid.cell_size) + pos,
            Vec3::new(0.0, 0.0, grid.half_cell_width as f32 * grid.cell_size) + pos,
            Color::RED,
        );

        //draw z central axis
        gizmos.line(
            Vec3::new(-grid.half_cell_width as f32 * grid.cell_size, 0.0, 0.0) + pos,
            Vec3::new(grid.half_cell_width as f32 * grid.cell_size, 0.0, 0.0) + pos,
            Color::BLUE,
        );
    }
}

fn auto_add_picking(
    mut commands: Commands,
    query: Query<Entity, (With<PrefabMarker>, Without<Pickable>)>,
) {
    for e in query.iter() {
        commands
            .entity(e)
            .insert(PickableBundle::default())
            .insert(On::<Pointer<Down>>::send_event::<SelectEvent>())
            .insert(RaycastPickable);
    }
}

type AutoAddQueryFilter = (
    Without<PrefabMarker>,
    Without<Pickable>,
    With<Parent>,
    Changed<Handle<Mesh>>,
);

//Auto add picking for each child to propagate picking event up to prefab entitiy
fn auto_add_picking_dummy(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>), AutoAddQueryFilter>,
    meshs: Res<Assets<Mesh>>,
) {
    for (e, mesh) in query.iter() {
        //Only meshed entity need to be pickable
        if let Some(mesh) = meshs.get(mesh) {
            if mesh.primitive_topology() == PrimitiveTopology::TriangleList {
                commands
                    .entity(e)
                    .insert(PickableBundle::default())
                    .insert(RaycastPickable);
            }
        }
    }
}

fn select_listener(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    // may need to be optimized a bit so that there is less overlap
    query_parent: Query<&SelectParent>,
    mut events: EventReader<SelectEvent>,
    pan_orbit_state: ResMut<PanOrbitEnabled>,
    keyboard: Res<Input<KeyCode>>,
) {
    if !pan_orbit_state.0 {
        return;
    }
    for event in events.read() {
        info!("Select Event: {:?}", event.e);
        let entity = match query_parent.get(event.e) {
            Ok(a) => a.parent,
            Err(_) => event.e,
        };
        match event.event.button {
            PointerButton::Primary => {
                commands.entity(entity).insert(Selected);
                if !keyboard.pressed(KeyCode::ShiftLeft) {
                    for e in query.iter() {
                        commands.entity(e).remove::<Selected>();
                    }
                }
            }
            PointerButton::Secondary => { /*Show context menu?*/ }
            PointerButton::Middle => {}
        }
    }
}

fn delete_selected(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let delete = keyboard.just_pressed(KeyCode::Back) || keyboard.just_pressed(KeyCode::Delete);

    if ctrl && shift && delete {
        for entity in query.iter() {
            info!("Delete Entity: {entity:?}");
            commands.entity(entity).despawn_recursive();
        }
    }
}

impl From<ListenerInput<Pointer<Down>>> for SelectEvent {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        Self {
            e: value.target(),
            event: value,
        }
    }
}

fn save_prefab_before_play(mut editor_events: EventWriter<space_shared::EditorEvent>) {
    editor_events.send(space_shared::EditorEvent::Save(
        space_shared::EditorPrefabPath::MemoryCahce,
    ));
}

fn to_game_after_save(mut state: ResMut<NextState<EditorState>>) {
    state.set(EditorState::Game);
}

fn set_start_state(mut state: ResMut<NextState<EditorState>>) {
    state.set(EditorState::Editor);
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

/// Resource, which contains pan orbit camera state
#[derive(Resource, Default)]
pub struct PanOrbitEnabled(pub bool);

/// This system executes before all UI systems and is used to enable pan orbit camera on frame start
pub fn reset_pan_orbit_state(mut state: ResMut<PanOrbitEnabled>) {
    *state = PanOrbitEnabled(true);
}

/// This system executes after all UI systems and is used to set pan orbit camera state.
/// For example, it will block pan orbit camera if pointer is used by egui
pub fn update_pan_orbit(
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    state: Res<PanOrbitEnabled>,
) {
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        pan_orbit.enabled = state.0;
    }
}

/// Sytem to block camera control if egui is using mouse
pub fn ui_camera_block(
    mut ctxs: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<PanOrbitEnabled>,
    game_view: Res<GameViewTab>,
) {
    let Ok(mut ctx_ref) = ctxs.get_single_mut() else {
        return;
    };
    let ctx = ctx_ref.get_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        let Some(pos) = ctx.pointer_latest_pos() else {
            return;
        };
        if let Some(area) = game_view.viewport_rect {
            if area.contains(pos) {
            } else {
                *state = PanOrbitEnabled(false);
            }
        } else {
            *state = PanOrbitEnabled(false);
        }
    }
}

type ChangeCameraQueryFilter = (Without<EditorCameraMarker>, With<CameraPlay>);

/// System to change camera from editor camera to game camera (if exist)
pub fn change_camera_in_play(
    mut cameras: Query<&mut Camera, (With<EditorCameraMarker>, Without<CameraPlay>)>,
    mut play_cameras: Query<(&mut Camera, &CameraPlay), ChangeCameraQueryFilter>,
) {
    if !play_cameras.is_empty() {
        let (mut some_camera, _) = play_cameras.iter_mut().next().unwrap();
        cameras.single_mut().is_active = false;
        some_camera.is_active = true;
    }
}

/// System to change camera from game camera to editor camera (if exist)
pub fn change_camera_in_editor(
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
    mut play_cameras: Query<&mut Camera, Without<EditorCameraMarker>>,
) {
    for mut ecam in cameras.iter_mut() {
        ecam.is_active = true;
    }

    for mut play_cam in play_cameras.iter_mut() {
        play_cam.is_active = false;
    }
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

fn draw_camera_gizmo(
    mut gizmos: Gizmos,
    cameras: Query<
        (&GlobalTransform, &Projection),
        (
            With<Camera>,
            Without<EditorCameraMarker>,
            Without<DisableCameraSkip>,
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

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        if !app.is_plugin_added::<MeshlessVisualizerPlugin>() {
            app.add_plugins(MeshlessVisualizerPlugin);
        }

        app.add_plugins((bottom_menu::BottomMenuPlugin, MouseCheck));

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
                    .before(ui_camera_block)
                    .after(bottom_menu::menu),
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
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());

        app.editor_tab_by_trait(
            EditorTabName::Other("Debug World Inspector".to_string()),
            self::debug_panels::DebugWorldInspector {},
        );

        app.add_plugins(CameraViewTabPlugin);

        app.add_plugins(SpaceHierarchyPlugin::default());
        app.add_plugins(SpaceInspectorPlugin);

        app.editor_tool(GizmoTool::default());
        app.add_plugins(GizmoToolPlugin);
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
pub fn simple_editor_setup(mut commands: Commands) {
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
        PrefabMarker,
        Name::new("Directional Light"),
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

    create_grid_lines(commands);
}
