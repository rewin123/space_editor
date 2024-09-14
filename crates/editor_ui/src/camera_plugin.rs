use crate::*;
use bevy::prelude::*;

pub struct EditorDefaultCameraPlugin;

impl Plugin for EditorDefaultCameraPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            reset_editor_camera_state
                .in_set(EditorSet::Editor)
                .before(UiSystemSet),
        );
        app.add_systems(
            Update,
            update_pan_orbit
                .after(reset_editor_camera_state)
                .before(PanOrbitCameraSystemSet)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(
            Update,
            ui_camera_block
                .after(reset_editor_camera_state)
                .before(update_pan_orbit)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(OnEnter(EditorState::GamePrepare), reset_play_camera_state);
        app.add_systems(OnEnter(EditorState::Editor), reset_editor_camera_state);
    }
}

/// Resource, which contains state for editor camera (default or any)
#[derive(Resource, Default)]
pub struct EditorCameraEnabled(pub bool);

impl From<bool> for EditorCameraEnabled {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// This system executes before all UI systems and is used to enable pan orbit camera on frame start
pub fn reset_editor_camera_state(mut state: ResMut<EditorCameraEnabled>) {
    *state = true.into();
}

/// This system executes before all UI systems and is used to enable pan orbit camera on frame start
pub fn reset_play_camera_state(mut state: ResMut<EditorCameraEnabled>) {
    *state = false.into();
}

/// This system executes after all UI systems and is used to set pan orbit camera state.
/// For example, it will block pan orbit camera if pointer is used by egui
pub fn update_pan_orbit(
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    state: Res<EditorCameraEnabled>,
) {
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        pan_orbit.enabled = state.0;
    }
}

type PlayModeCameraFilter = (Without<EditorCameraMarker>, With<PlaymodeCamera>);
type EditorModeCameraFilter = (With<EditorCameraMarker>, Without<PlaymodeCamera>);

/// System to change camera from editor camera to game play camera (if exist)
pub fn change_camera_in_play(
    mut editor_cameras: Query<&mut Camera, EditorModeCameraFilter>,
    mut play_cameras: Query<&mut Camera, PlayModeCameraFilter>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut toast: EventWriter<ToastMessage>,
) {
    if !play_cameras.is_empty() {
        editor_cameras.iter_mut().for_each(|mut cam| {
            cam.is_active = false;
        });
        play_cameras.iter_mut().for_each(|mut cam| {
            cam.is_active = true;
        });

        let Ok(window) = primary_window.get_single() else {
            error!("Failed to get Primary Window");
            toast.send(ToastMessage::new(
                "Failed to get Primary Window",
                space_shared::toast::ToastKind::Error,
            ));
            return;
        };
        let Ok(mut cam) = play_cameras.get_single_mut() else {
            error!("No play camera found");
            toast.send(ToastMessage::new(
                "No play camera found",
                space_shared::toast::ToastKind::Error,
            ));
            return;
        };
        cam.viewport = Some(bevy::render::camera::Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.width() as u32, window.height() as u32),
            depth: 0.0..1.0,
        });
    } else {
        error!("No play camera found");
        toast.send(ToastMessage::new(
            "No play camera found",
            space_shared::toast::ToastKind::Error,
        ));
    }
}

/// System to change camera from game camera to editor camera (if exist)
pub fn change_camera_in_editor(
    mut editor_cameras: Query<&mut Camera, EditorModeCameraFilter>,
    mut play_cameras: Query<&mut Camera, PlayModeCameraFilter>,
) {
    for mut ecam in editor_cameras.iter_mut() {
        ecam.is_active = true;
    }

    for mut play_cam in play_cameras.iter_mut() {
        play_cam.is_active = false;
    }
}

///Camera with this component will not be disabled in Editor state
#[derive(Component)]
pub struct DisableCameraSkip;

pub fn disable_no_editor_cams(
    mut cameras: Query<&mut Camera, (Without<DisableCameraSkip>, Without<EditorCameraMarker>)>,
) {
    for mut cam in cameras.iter_mut() {
        cam.is_active = false;
    }
}

#[derive(Component)]
pub struct NotShowCamera;

pub fn draw_camera_gizmo(
    mut gizmos: Gizmos<EditorGizmo>,
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
        let pink = Color::srgb(1.0, 0.41, 0.71);

        let transform = transform.compute_transform();
        let cuboid_transform = transform.with_scale(Vec3::new(1.0, 1.0, 2.0));
        gizmos.cuboid(cuboid_transform, pink);

        let scale = 1.5;

        gizmos.line(
            transform.translation,
            transform.translation
                + transform.forward() * scale
                + transform.up() * scale
                + transform.right() * scale,
            pink,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale - transform.up() * scale
                + transform.right() * scale,
            pink,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale + transform.up() * scale
                - transform.right() * scale,
            pink,
        );
        gizmos.line(
            transform.translation,
            transform.translation + transform.forward() * scale
                - transform.up() * scale
                - transform.right() * scale,
            pink,
        );

        let rect_transform = Transform::from_xyz(0.0, 0.0, -scale);
        let rect_transform = transform.mul_transform(rect_transform);

        gizmos.rect(
            rect_transform.translation,
            rect_transform.rotation,
            Vec2::splat(scale * 2.0),
            pink,
        );
    }
}
