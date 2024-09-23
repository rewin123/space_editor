#![allow(clippy::type_complexity)]

/// This module contains ui logics, which will be work through events with editor core module and prefab module
mod mouse_check;

/// This module will be used to create Unity like project file dialog. Currently NOT USED
pub mod asset_inspector;

/// This module contains logic for menu toolbars
pub mod menu_toolbars;

/// This module contains UI logic for undo/redo functionality
pub mod change_chain;

/// This module contains UI logic for debug panels (like WorldInspector)
pub mod debug_panels;

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

/// Editor tab name
pub mod editor_tab_name;

pub mod colors;
/// This module contains editor style definitions
pub mod editor_style;
pub mod sizing;

pub mod icons;

use bevy_debug_grid::{Grid, GridAxis, SubGrid, TrackedGrid};
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
use space_editor_core::prelude::*;

use bevy::{
    app::PluginGroupBuilder,
    input::common_conditions::input_toggle_active,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{render_resource::PrimitiveTopology, view::RenderLayers},
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext};

use space_editor_tabs::prelude::*;

use game_view::{has_window_changed, GameViewPlugin};
use prelude::{
    clean_meshless, reset_camera_viewport, set_camera_viewport, GameModeSettings, GameViewTab,
    MeshlessVisualizerPlugin, SpaceHierarchyPlugin, SpaceInspectorPlugin,
};
use space_editor_core::toast::ToastUiPlugin;
use space_prefab::prelude::*;
use space_shared::{
    ext::bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin},
    toast::ToastMessage,
    EditorCameraMarker, EditorSet, EditorState, PrefabMarker, PrefabMemoryCache,
};
use space_undo::{SyncUndoMarkersPlugin, UndoPlugin, UndoSet};
use ui_registration::BundleReg;

use camera_plugin::*;
use ui_plugin::*;

use self::mouse_check::MouseCheck; // , tools::gizmo::GizmoToolPlugin};

pub const MAX_RENDER_LAYERS: u8 = 32; // Or however many layers you need

pub const DEFAULT_GRID_ALPHA: f32 = 0.5_f32;

pub fn all_render_layers() -> RenderLayers {
    (0..MAX_RENDER_LAYERS).fold(RenderLayers::none(), |layers, layer| {
        layers.with(layer.into())
    })
}

pub const LAST_RENDER_LAYER: u8 = MAX_RENDER_LAYERS - 1;

pub mod prelude {
    pub use super::{
        asset_inspector::*,
        change_chain::*,
        debug_panels::*,
        game_view::*,
        hierarchy::*,
        inspector::*,
        menu_toolbars::*,
        meshless_visualizer::*,
        settings::*,
        tool::*, //tools::*,
        ui_registration::*,
    };

    pub use space_editor_core::prelude::*;
    pub use space_editor_tabs::prelude::*;
    pub use space_persistence::*;
    pub use space_prefab::prelude::*;
    pub use space_shared::prelude::*;

    pub use crate::camera_plugin::*;
    pub use crate::selection::*;
    pub use crate::simple_editor_setup;
    pub use crate::ui_plugin::*;
    pub use crate::EditorPlugin;

    pub use crate::editor_tab_name::*;
}

/// External dependencies for editor crate
pub mod ext {
    pub use bevy_egui;
    pub use bevy_mod_picking;
    pub use bevy_panorbit_camera;
    pub use space_shared::ext::*;
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    #[cfg(not(tarpaulin_include))]
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
            .add(EditorGizmoPlugin)
            .add(ToastUiPlugin)
            .add(UndoPlugin)
            .add(SyncUndoMarkersPlugin::<PrefabMarker>::default())
            .add(PrefabPlugin)
            .add(space_editor_core::EditorCore)
            .add(EditorSetsPlugin)
            .add(EditorDefaultBundlesPlugin)
            .add(EditorDefaultCameraPlugin)
            .add(bevy_egui::EguiPlugin)
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
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        ui_registration::register_mesh_editor_bundles(app);
        ui_registration::register_light_editor_bundles(app);
    }
}

pub struct EditorSetsPlugin;

impl Plugin for EditorSetsPlugin {
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.add_systems(Update, editor_gizmos);
        app.add_systems(Update, game_gizmos);
    }
}

fn editor_gizmos(mut gizmos_config: ResMut<GizmoConfigStore>) {
    gizmos_config.config_mut::<EditorGizmo>().0.render_layers =
        RenderLayers::layer(LAST_RENDER_LAYER.into())
}

fn game_gizmos(mut gizmos_config: ResMut<GizmoConfigStore>) {
    gizmos_config.config_mut::<EditorGizmo>().0.render_layers = RenderLayers::layer(0)
}

type AutoAddQueryFilter = (
    Without<PrefabMarker>,
    Without<Pickable>,
    With<Parent>,
    Changed<Handle<Mesh>>,
);

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct EditorGizmo;

pub struct EditorGizmoPlugin;

impl Plugin for EditorGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmo>();
    }
}

fn save_prefab_before_play(
    mut editor_events: EventWriter<space_shared::EditorEvent>,
    mut toast: EventWriter<ToastMessage>,
) {
    toast.send(ToastMessage::new(
        "Preparing prefab to save for playmode",
        space_shared::toast::ToastKind::Info,
    ));
    editor_events.send(space_shared::EditorEvent::Save(
        space_shared::EditorPrefabPath::MemoryCache,
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
    match save_confg.path.as_ref() {
        Some(space_shared::EditorPrefabPath::File(path)) => {
            info!("Loading prefab from file {}", path);
            load_server.scene = Some(assets.load(format!("{}.scn.ron", path)));
        }
        Some(space_shared::EditorPrefabPath::MemoryCache) => {
            info!("Loading prefab from cache");
            load_server.scene.clone_from(&cache.scene);
        }
        _ => {}
    }
}

pub trait FlatPluginList {
    fn add_plugins_to_group(&self, group: PluginGroupBuilder) -> PluginGroupBuilder;
}

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
        Name::from("Editor Level Light"),
    ));

    // grid
    let grid_render_layer = RenderLayers::layer(LAST_RENDER_LAYER.into());
    let silver = Color::srgb(0.75, 0.75, 0.75);
    let grey = Color::srgb(0.5, 0.5, 0.5);

    commands.spawn((
        Grid {
            spacing: 10.0_f32,
            count: 16,
            color: silver.with_alpha(DEFAULT_GRID_ALPHA),
            alpha_mode: AlphaMode::Blend,
        },
        SubGrid {
            count: 9,
            color: grey.with_alpha(DEFAULT_GRID_ALPHA),
        },
        // Darker grid to make it easier to see entity gizmos when Transform (0, 0, 0)
        GridAxis {
            x: Some(Color::linear_rgb(0.9, 0.1, 0.1)),
            y: Some(Color::linear_rgb(0.1, 0.9, 0.1)),
            z: Some(Color::linear_rgb(0.1, 0.1, 0.9)),
        },
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
                order: 100,
                ..default()
            },
            ..default()
        },
        bevy_panorbit_camera::PanOrbitCamera::default(),
        EditorCameraMarker,
        Name::from("Editor Camera"),
        PickableBundle::default(),
        RaycastPickable,
        all_render_layers(),
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
                        // We had too many editor cameras at order 0
                        order: 100,
                        ..default()
                    },
                    ..default()
                },
                bevy_panorbit_camera::PanOrbitCamera::default(),
                EditorCameraMarker,
                Name::from("Editor Camera"),
                PickableBundle::default(),
                RaycastPickable,
                all_render_layers(),
            ));
        } else {
            // 2D camera
            commands.spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: 100,
                        ..default()
                    },
                    ..default()
                },
                EditorCameraMarker,
                Name::from("Editor 2D Camera"),
                PickableBundle::default(),
                RaycastPickable,
                all_render_layers(),
            ));
        }
    }
}
