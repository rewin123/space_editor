//code only for editor gui

use bevy::{prelude::*, window::PrimaryWindow, render::render_resource::PrimitiveTopology};

pub mod core;
pub mod ui;

pub mod ui_registration;

use bevy_egui::{EguiContext, EguiContexts};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin};
use bevy_mod_picking::{prelude::*, PickableBundle, backends::raycast::{bevy_mod_raycast::prelude::RaycastSettings, RaycastPickable}};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};

use crate::{
    prefab::{component::CameraPlay, save::SaveState},
    prelude::GameViewTab,
    EditorCameraMarker, EditorSet, EditorState, PrefabMarker,
};

use ui_registration::*;

use self::prelude::{EditorUiPlugin, UiSystemSet};

/// All useful structs and functions from editor UI
pub mod prelude {
    pub use super::ui::*;
}

/// Editor UI plugin. Must be used with [`PrefabPlugin`] and [`EditorRegistryPlugin`]
///
/// [`PrefabPlugin`]: crate::PrefabPlugin
/// [`EditorRegistryPlugin`]: crate::editor_registry::EditorRegistryPlugin
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }

        app.add_plugins(EventListenerPlugin::<SelectEvent>::default());

        app.add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(EditorUiPlugin::default())
            .add_plugins(PanOrbitCameraPlugin);

        if !app.is_plugin_added::<bevy_mod_picking::prelude::SelectionPlugin>() {
            app.add_plugins(
                bevy_mod_picking::DefaultPickingPlugins
            );

            app.world.resource_mut::<backends::raycast::RaycastBackendSettings>().require_markers = true;
        }

        if !app.is_plugin_added::<bevy_debug_grid::DebugGridPlugin>() {
            app.add_plugins(bevy_debug_grid::DebugGridPlugin::without_floor_grid());
        }
        app.init_resource::<prelude::EditorLoader>();

        app.insert_resource(PanOrbitEnabled(true));

        app.add_plugins(core::EditorCore);

        app.add_systems(
            Startup,
            (set_start_state, apply_state_transition::<EditorState>)
                .chain()
                .in_set(EditorSet::Editor),
        );

        app.add_systems(Update, reset_pan_orbit_state.in_set(EditorSet::Editor));
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

        app.add_systems(
            OnEnter(EditorState::Editor),
            (
                clear_and_load_on_start,
                change_camera_in_editor,
                create_grid_lines,
            ),
        );

        app.add_systems(
            PostUpdate,
            (
                auto_add_picking,
                select_listener.after(UiSystemSet),
            )
                .run_if(in_state(EditorState::Editor)),
        );
        app.add_systems(
            PostUpdate,
            auto_add_picking_dummy);

        app.add_systems(
            Update,
            (draw_camera_gizmo, disable_no_editor_cams).run_if(in_state(EditorState::Editor)),
        );

        app.add_event::<SelectEvent>();

        app.init_resource::<BundleReg>();

        app.add_plugins(WorldInspectorPlugin::default().run_if(in_state(EditorState::Game)));

        register_mesh_editor_bundles(app);
        register_light_editor_bundles(app);
    }
}

#[derive(Event, Clone, EntityEvent)]
struct SelectEvent {
    #[target]
    e: Entity,
    event: ListenerInput<Pointer<Down>>,
}

fn create_grid_lines(mut commands: Commands) {
    bevy_debug_grid::spawn_floor_grid(commands);
}

fn cleanup_grid_lines(mut commands: Commands, query: Query<Entity, With<bevy_debug_grid::Grid>>) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
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
    Changed<Handle<Mesh>>);

//Auto add picking for each child to propagate picking event up to prefab entitiy
fn auto_add_picking_dummy(
        mut commands: Commands, 
        query : Query<(Entity, &Handle<Mesh>), AutoAddQueryFilter>,
        meshs : Res<Assets<Mesh>>) {
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
    query: Query<Entity, With<core::Selected>>,
    mut events: EventReader<SelectEvent>,
    _ctxs: EguiContexts,
    pan_orbit_state: ResMut<PanOrbitEnabled>,
    keyboard: Res<Input<KeyCode>>,
) {
    if !pan_orbit_state.0 {
        return;
    }
    for event in events.read() {
        info!("Select Event: {:?}", event.e);
        match event.event.button {
            PointerButton::Primary => {
                commands.entity(event.e).insert(core::Selected);
                if !keyboard.pressed(KeyCode::ShiftLeft) {
                    for e in query.iter() {
                        commands.entity(e).remove::<core::Selected>();
                    }
                }
            }
            PointerButton::Secondary => { /*Show context menu?*/ }
            PointerButton::Middle => {}
        }
    }
}

impl From<ListenerInput<Pointer<Down>>> for SelectEvent {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        SelectEvent {
            e: value.target(),
            event: value,
        }
    }
}

fn save_prefab_before_play(mut editor_events: EventWriter<core::EditorEvent>) {
    editor_events.send(core::EditorEvent::Save(core::EditorPrefabPath::MemoryCahce));
}

fn to_game_after_save(mut state: ResMut<NextState<EditorState>>) {
    state.set(EditorState::Game);
}

fn set_start_state(mut state: ResMut<NextState<EditorState>>) {
    state.set(EditorState::Editor);
}

fn clear_and_load_on_start(
    mut load_server: ResMut<prelude::EditorLoader>,
    save_confg: Res<crate::prefab::save::SaveConfig>,
    assets: Res<AssetServer>,
    cache: Res<core::PrefabMemoryCache>,
) {
    if save_confg.path.is_none() {
        return;
    }
    match save_confg.path.as_ref().unwrap() {
        core::EditorPrefabPath::File(path) => {
            info!("Loading prefab from file {}", path);
            load_server.scene = Some(assets.load(format!("{}.scn.ron", path)));
        }
        core::EditorPrefabPath::MemoryCahce => {
            info!("Loading prefab from cache");
            load_server.scene = cache.scene.clone();
        }
    }
}

/// Resource, which contains pan orbit camera state
#[derive(Resource, Default)]
pub struct PanOrbitEnabled(pub bool);

pub fn reset_pan_orbit_state(mut state: ResMut<PanOrbitEnabled>) {
    *state = PanOrbitEnabled(true);
}

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

fn disable_no_editor_cams(mut cameras: Query<&mut Camera, Without<EditorCameraMarker>>) {
    for mut cam in cameras.iter_mut() {
        cam.is_active = false;
    }
}

fn draw_camera_gizmo(
    mut gizom: Gizmos,
    cameras: Query<(&GlobalTransform, &Projection), (With<Camera>, Without<EditorCameraMarker>)>,
) {
    for (transform, _projection) in cameras.iter() {
        let transform = transform.compute_transform();
        let cuboid_transform = transform.with_scale(Vec3::new(1.0, 1.0, 2.0));
        gizom.cuboid(cuboid_transform, Color::PINK);

        let scale = 1.5;

        gizom.line(
            transform.translation,
            transform.translation
                + transform.forward() * scale
                + transform.up() * scale
                + transform.right() * scale,
            Color::PINK,
        );
        gizom.line(
            transform.translation,
            transform.translation + transform.forward() * scale - transform.up() * scale
                + transform.right() * scale,
            Color::PINK,
        );
        gizom.line(
            transform.translation,
            transform.translation + transform.forward() * scale + transform.up() * scale
                - transform.right() * scale,
            Color::PINK,
        );
        gizom.line(
            transform.translation,
            transform.translation + transform.forward() * scale
                - transform.up() * scale
                - transform.right() * scale,
            Color::PINK,
        );

        let rect_transform = Transform::from_xyz(0.0, 0.0, -scale);
        let rect_transform = transform.mul_transform(rect_transform);

        gizom.rect(
            rect_transform.translation,
            rect_transform.rotation,
            Vec2::splat(scale * 2.0),
            Color::PINK,
        );
    }
}

// fn auto_remove_pickable(
//     mut commands : Commands,
//     grids : Query<Entity, (With<Pickable>, Or<(With<bevy_debug_grid::Grid>, With<bevy_debug_grid::GridChild>)>)>
// ) {
//     for e in grids.iter() {
//         commands.entity(e).remove::<Pickable>();
//     }
// }