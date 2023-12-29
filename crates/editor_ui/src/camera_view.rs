use bevy::{
    prelude::*,
    render::{
        camera::{RenderTarget, TemporalJitter},
        view::RenderLayers,
    },
    window::{PrimaryWindow, WindowRef},
};
use bevy_egui::egui::{self};

use space_shared::*;

use crate::{prelude::EditorTabName, DisableCameraSkip, EditorUiAppExt};

use super::editor_tab::EditorTab;

pub struct CameraViewTabPlugin;

impl Plugin for CameraViewTabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::CameraView, CameraViewTab::default());
        app.add_systems(PostUpdate, set_camera_viewport.in_set(EditorSet::Editor));
        app.add_systems(OnEnter(EditorState::GamePrepare), clean_camera_view_tab);
    }
}

#[derive(Component)]
pub struct ViewCamera;

/// Tab for camera view in editor
#[derive(Resource, Default)]
pub struct CameraViewTab {
    pub viewport_rect: Option<egui::Rect>,
    pub camera_entity: Option<Entity>,
    pub real_camera: Option<Entity>,
}

impl EditorTab for CameraViewTab {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, commands: &mut Commands, world: &mut World) {
        if self.real_camera.is_none() {
            self.real_camera = Some(
                commands
                    .spawn((
                        Camera3dBundle {
                            camera: Camera {
                                is_active: false,
                                order: 2,
                                ..default()
                            },
                            camera_3d: Camera3d {
                                clear_color:
                                    bevy::core_pipeline::clear_color::ClearColorConfig::None,
                                ..default()
                            },
                            ..default()
                        },
                        RenderLayers::layer(0),
                        TemporalJitter::default(),
                        Name::new("Camera for Camera view tab"),
                        DisableCameraSkip,
                        ViewCamera,
                    ))
                    .id(),
            );
        }

        let mut camera_query = world.query_filtered::<Entity, (
            With<Camera>,
            Without<EditorCameraMarker>,
            Without<ViewCamera>,
        )>();

        egui::ComboBox::from_label("Camera")
            .selected_text(format!("{:?}", self.camera_entity))
            .show_ui(ui, |ui| {
                for entity in camera_query.iter(world) {
                    ui.selectable_value(
                        &mut self.camera_entity,
                        Some(entity),
                        format!("{:?}", entity),
                    );
                }
            });

        ui.add_space(4.);
        ui.separator();

        // Moves camera below the selection
        let pos = ui.next_widget_position();
        let mut clipped = ui.clip_rect();
        clipped.set_left(pos.x);
        clipped.set_top(pos.y);
        self.viewport_rect = Some(clipped);
    }

    fn title(&self) -> bevy_egui::egui::WidgetText {
        "Camera view".into()
    }
}

fn clean_camera_view_tab(
    mut ui_state: ResMut<CameraViewTab>,
    mut cameras: Query<(&mut Camera, &mut Transform), Without<EditorCameraMarker>>,
) {
    let Some(real_cam_entity) = ui_state.real_camera else {
        return;
    };

    let Ok((mut real_cam, _real_cam_transform)) = cameras.get_mut(real_cam_entity) else {
        return;
    };

    real_cam.is_active = false;
    real_cam.viewport = None;

    ui_state.camera_entity = None;
    ui_state.real_camera = None;
    ui_state.viewport_rect = None;
}

#[derive(Default)]
struct LastCamTabRect(Option<egui::Rect>);

fn set_camera_viewport(
    mut local: Local<LastCamTabRect>,
    ui_state: Res<CameraViewTab>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<(&mut Camera, &mut Transform), Without<EditorCameraMarker>>,
) {
    let Some(real_cam_entity) = ui_state.real_camera else {
        return;
    };

    let Some(camera_entity) = ui_state.camera_entity else {
        return;
    };

    let Ok([(mut real_cam, mut real_cam_transform), (watch_cam, camera_transform)]) =
        cameras.get_many_mut([real_cam_entity, camera_entity])
    else {
        return;
    };

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(viewport_rect) = ui_state.viewport_rect else {
        return;
    };

    local.0 = Some(viewport_rect);

    if watch_cam.is_changed() {
        *real_cam = watch_cam.clone();
    }
    real_cam.order = 2;
    real_cam.is_active = true;

    *real_cam_transform = *camera_transform;

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let mut viewport_size = viewport_rect.size() * scale_factor as f32;

    // Fixes camera viewport size to be proportional to main watch camera
    if let Some(watch_cam_size) = watch_cam.logical_viewport_size() {
        let wx = watch_cam_size.x;
        let wy = watch_cam_size.y;

        viewport_size.y = wy * viewport_size.x / wx
    }

    real_cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });

    real_cam.target = RenderTarget::Window(WindowRef::Primary);
}
