//code only for editor gui

use bevy::prelude::*;

/// Contains all component inspector login
pub mod inspector;
/// Contains all bot panel logic
pub mod bot_menu;
/// Contains all hierarchy panel logic
pub mod hierarchy;
/// Contains logic for selecting entities
pub mod selected;
/// Not used riught now. Planned to be asset inspector UI for all assets in asset/ folder
pub mod asset_insector;
/// Contains logic to register editor bundles for fast spawning entities with fixed components set
pub mod ui_registration;

use bevy_egui::EguiContexts;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGrid};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, quick::WorldInspectorPlugin};
use bevy_mod_picking::{prelude::*, PickableBundle};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};

use crate::{EditorState, EditorSet, prefab::{save::SaveState, component::CameraPlay}, PrefabMarker, EditorCameraMarker};

use self::prelude::Selected;

use ui_registration::*;

/// All useful structs and functions from editor UI
pub mod prelude {
    pub use super::inspector::*;
    pub use super::bot_menu::*;
    pub use super::hierarchy::*;
    pub use super::selected::*;
    pub use bevy_panorbit_camera::{PanOrbitCamera};
}

/// Editor UI plugin. Must be used with PrefabPlugin and EditorRegistryPlugin
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }
        
        app
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(prelude::SpaceHierarchyPlugin::default())
        .add_plugins(prelude::InspectorPlugin)
        .add_plugins(prelude::BotMenuPlugin)
        .add_plugins(PanOrbitCameraPlugin);

        if !app.is_plugin_added::<bevy_mod_picking::prelude::SelectionPlugin>() {
            app.add_plugins(bevy_mod_picking::DefaultPickingPlugins.build()
                .disable::<DebugPickingPlugin>()
                .disable::<DefaultHighlightingPlugin>());
        }

        if !app.is_plugin_added::<bevy_infinite_grid::InfiniteGridPlugin>() {
            app.add_plugins(bevy_infinite_grid::InfiniteGridPlugin);
        }

        app.insert_resource(PanOrbitEnabled(true));

        app.add_systems(Startup, (set_start_state, apply_state_transition::<EditorState>).chain().in_set(EditorSet::Editor));

        app.add_systems(Update, reset_pan_orbit_state
            .in_set(EditorSet::Editor));
        app.add_systems(Update, update_pan_orbit
            .after(reset_pan_orbit_state)
            .before(PanOrbitCameraSystemSet)
            .in_set(EditorSet::Editor));
        app.add_systems(Update, ui_camera_block.after(reset_pan_orbit_state).
            before(update_pan_orbit)
            .in_set(EditorSet::Editor));

        //play systems
        app.add_systems(OnEnter(EditorState::GamePrepare), (cleanup_grid_lines, save_prefab_before_play));
        app.add_systems(OnEnter(SaveState::Idle), to_game_after_save.run_if(in_state(EditorState::GamePrepare)));

        app.add_systems(OnEnter(EditorState::Game), change_camera_in_play);

        app.add_systems(OnEnter(EditorState::Editor), 
            (clear_and_load_on_start, change_camera_in_editor, create_grid_lines));

        app.add_systems(PostUpdate, 
            (auto_add_picking, 
                select_listener,
                auto_add_picking_dummy)
                .run_if(in_state(EditorState::Editor)));

        app.add_systems(Update, 
            (draw_camera_gizmo, 
                disable_no_editor_cams).run_if(in_state(EditorState::Editor)));

        app.add_event::<SelectEvent>();

        app.init_resource::<EditorUiReg>();

        app.add_plugins(WorldInspectorPlugin::default().run_if(in_state(EditorState::Game)));

        register_default_editor_bundles(app);
    }
}



#[derive(Event)]
struct SelectEvent {
    e : Entity,
    event : ListenerInput<Pointer<Down>>
}

fn create_grid_lines(
    mut commands : Commands,
) {
    commands.spawn(InfiniteGridBundle::default());
}

fn cleanup_grid_lines(
    mut commands : Commands,
    query : Query<Entity, With<InfiniteGrid>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn auto_add_picking(
    mut commands : Commands,
    query : Query<Entity, (With<PrefabMarker>, Without<Pickable>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Down>>::send_event::<SelectEvent>());
    }
}

fn auto_add_picking_dummy(
    mut commands : Commands,
    query : Query<Entity, (Without<PrefabMarker>, Without<Pickable>, With<Parent>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(PickableBundle::default())
            .insert(RaycastPickTarget::default());
    }
}

fn select_listener(
    mut commands : Commands,
    query : Query<Entity, With<Selected>>,
    mut events : EventReader<SelectEvent>,
    mut ctxs : EguiContexts,
    keyboard: Res<Input<KeyCode>>,
) {
    if ctxs.ctx_mut().is_pointer_over_area() || ctxs.ctx_mut().is_using_pointer() {
        return;
    }
    for event in events.iter() {
        match event.event.button {
            PointerButton::Primary => {
                commands.entity(event.e).insert(Selected);
                if !keyboard.pressed(KeyCode::ShiftLeft) {
                    for e in query.iter() {
                        commands.entity(e).remove::<Selected>();
                    }
                }
            },
            PointerButton::Secondary => {/*Show context menu?*/},
            PointerButton::Middle => {},
        }
    }
}

impl From<ListenerInput<Pointer<Down>>> for SelectEvent {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        SelectEvent { e: value.listener(), event: value }
    }
}

fn save_prefab_before_play(
    mut save_state : ResMut<NextState<SaveState>>,
) {
    save_state.set(SaveState::Save);
}

fn to_game_after_save(
    mut state : ResMut<NextState<EditorState>>
) {
    state.set(EditorState::Game);
}

fn set_start_state(
    mut state : ResMut<NextState<EditorState>>
) {
    state.set(EditorState::Editor);
}

fn clear_and_load_on_start(
    mut load_server : ResMut<prelude::EditorLoader>,
    save_confg : Res<crate::prefab::save::SaveConfig>,
    assets : Res<AssetServer>,
) {
    if save_confg.path.is_empty() {
        return;
    }
    load_server.scene = Some(
        assets.load(format!("{}.scn.ron",save_confg.path))
    );
}

/// Resource, which contains pan orbit camera state
#[derive(Resource, Default)]
pub struct PanOrbitEnabled(pub bool);


pub fn reset_pan_orbit_state(
    mut state : ResMut<PanOrbitEnabled>
) {
    *state = PanOrbitEnabled(true);
}

pub fn update_pan_orbit(
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    state : Res<PanOrbitEnabled>,
) {
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        pan_orbit.enabled = state.0;
    }    
}


/// Sytem to block camera control if egui is using mouse 
pub fn ui_camera_block(
    mut ctxs : EguiContexts,
    mut state : ResMut<PanOrbitEnabled>
) {
    if ctxs.ctx_mut().is_pointer_over_area() || ctxs.ctx_mut().is_using_pointer() {
        *state = PanOrbitEnabled(false);
    }
}

/// System to change camera from editor camera to game camera (if exist)
pub fn change_camera_in_play(
    mut cameras : Query<(&mut Camera), (With<EditorCameraMarker>, Without<CameraPlay>)>,
    mut play_cameras : Query<(&mut Camera, &CameraPlay), (Without<EditorCameraMarker>, With<CameraPlay>)>
) {
    if !play_cameras.is_empty() {
        let (mut some_camera, _) = play_cameras.iter_mut().next().unwrap();
        cameras.single_mut().is_active = false;
        some_camera.is_active = true;
    }
}

/// System to change camera from game camera to editor camera (if exist)
pub fn change_camera_in_editor(
    mut cameras : Query<(&mut Camera), (With<EditorCameraMarker>)>,
    mut play_cameras : Query<&mut Camera, (Without<EditorCameraMarker>)>
) {
    for mut ecam in cameras.iter_mut() {
        ecam.is_active = true;
    }

    for mut play_cam in play_cameras.iter_mut() {
        play_cam.is_active = false;
    }
}

fn disable_no_editor_cams(
    mut cameras : Query<(&mut Camera), (Without<EditorCameraMarker>)>,
) {
    for mut cam in cameras.iter_mut() {
        cam.is_active = false;
    }
}

fn draw_camera_gizmo(
    mut gizom : Gizmos,
    cameras : Query<(&GlobalTransform, &Projection), (With<Camera>, Without<EditorCameraMarker>)>
) {
    for (transform, projection) in cameras.iter() {
        let transform = transform.compute_transform();
        let cuboid_transform = transform.with_scale(Vec3::new(1.0, 1.0, 2.0));
        gizom.cuboid(cuboid_transform, Color::PINK);

        let scale = 1.5;
        
        gizom.line(transform.translation, transform.translation + transform.forward() * scale + transform.up() * scale + transform.right() * scale, Color::PINK);
        gizom.line(transform.translation, transform.translation + transform.forward() * scale - transform.up() * scale + transform.right() * scale, Color::PINK);
        gizom.line(transform.translation, transform.translation + transform.forward() * scale + transform.up() * scale - transform.right() * scale, Color::PINK);
        gizom.line(transform.translation, transform.translation + transform.forward() * scale - transform.up() * scale - transform.right() * scale, Color::PINK);

        let rect_transform = Transform::from_xyz(0.0, 0.0, -scale);
        let rect_transform = transform.mul_transform(rect_transform);

        gizom.rect(rect_transform.translation, rect_transform.rotation, Vec2::splat(scale * 2.0), Color::PINK);
    }
}