use crate::*;
use bevy::prelude::*;

pub struct EditorDefaultCameraPlugin;

impl Plugin for EditorDefaultCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reset_editor_camera_state.before(UiSystemSet::Init));
        app.add_systems(Update, update_pan_orbit.in_set(UiSystemSet::Last));

        app.add_systems(Update, ui_camera_block.in_set(UiSystemSet::AfterShow));

        app.configure_sets(Update, UiSystemSet::Last.before(PanOrbitCameraSystemSet));
    }
}

/// Resource, which contains state for editor camera (default or any)
#[derive(Resource, Default)]
pub struct EditorCameraEnabled(pub bool);

/// This system executes before all UI systems and is used to enable pan orbit camera on frame start
pub fn reset_editor_camera_state(mut state: ResMut<EditorCameraEnabled>) {
    *state = EditorCameraEnabled(true);
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
