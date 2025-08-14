use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use ext::bevy_inspector_egui::quick::WorldInspectorPlugin;
use space_editor::prelude::*;

use space_editor_game_view::gizmo_tool;
use space_editor_ui::ext::bevy_panorbit_camera::PanOrbitCamera;
use transform_gizmo_bevy::{mouse_interact::MouseGizmoInteractionPlugin, picking::TransformGizmoPickingPlugin, GizmoCamera, GizmoTarget, TransformGizmoPlugin};

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::DEBUG,
            ..default()
        }))
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)

        .add_plugins(TransformGizmoPlugin)

        .add_systems(
            Update,
            disable_pan_orbit_on_gizmo
                .after(update_pan_orbit)
                .in_set(EditorSet::Editor),
        )

        .register_type::<GizmoCamera>()
        .register_type::<GizmoTarget>()
        .editor_registry::<GizmoCamera>()
        .editor_registry::<GizmoTarget>()
        .register_type::<PanOrbitCamera>()

        .run();
}


fn disable_pan_orbit_on_gizmo(
    mut pan_orbit_cams: Query<&mut PanOrbitCamera, With<GizmoCamera>>,
    gizmo_targets: Query<&GizmoTarget>,
) {
    for mut cam in pan_orbit_cams.iter_mut() {
        for gizmo_target in gizmo_targets.iter() {
            if gizmo_target.is_active() {
                cam.enabled = false;
                //debug!("Disabling PanOrbitCamera for GizmoTarget: {:?}", gizmo_target);
                return;
            }
        }
    }
}
