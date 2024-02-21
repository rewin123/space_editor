use bevy::{
    prelude::*,
    render::{
        camera::{RenderTarget, TemporalJitter},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::PrimaryWindow,
};
use bevy_egui_next::{egui, EguiContexts};

use space_prefab::component::CameraPlay;
use space_shared::*;

use crate::{
    colors::ERROR_COLOR,
    prelude::{EditorTabName, GameModeSettings},
    DisableCameraSkip, EditorUiAppExt, RenderLayers, UiSystemSet,
};

use super::editor_tab::EditorTab;

pub struct CameraViewTabPlugin;

impl Plugin for CameraViewTabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::CameraView, CameraViewTab::default());
        app.add_systems(
            Update,
            set_camera_viewport
                .after(UiSystemSet::Last)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(OnEnter(EditorState::Game), clean_camera_view_tab);
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
    pub target_image: Option<Handle<Image>>,
    pub egui_tex_id: Option<(egui::TextureId, Handle<Image>)>,
    pub need_reinit_egui_tex: bool,
}

fn create_camera_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    image
}

impl EditorTab for CameraViewTab {
    fn ui(
        &mut self,
        ui: &mut bevy_egui_next::egui::Ui,
        commands: &mut Commands,
        world: &mut World,
    ) {
        if self.real_camera.is_none() {
            if world.resource::<GameModeSettings>().is_3d() {
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
                                        bevy::core_pipeline::clear_color::ClearColorConfig::Default,
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
            } else if world.resource::<GameModeSettings>().is_2d() {
                self.real_camera = Some(
                    commands
                        .spawn((
                            Camera2dBundle::default(),
                            RenderLayers::layer(0),
                            Name::new("Camera for Camera view tab"),
                            DisableCameraSkip,
                            ViewCamera,
                        ))
                        .id(),
                );
            }
        }

        let mut camera_query = world.query_filtered::<Entity, (
            With<Camera>,
            Without<EditorCameraMarker>,
            Without<ViewCamera>,
        )>();

        if camera_query.iter(world).count() == 1 {
            let selected_entity = camera_query.iter(world).next();
            self.camera_entity = selected_entity;
            ui.label(format!("Camera: {:?}", selected_entity.unwrap()));
        } else if camera_query.iter(world).count() > 0 {
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
            ui.spacing();
            ui.separator();
        } else {
            ui.label(egui::RichText::new("No available Cameras").color(ERROR_COLOR));

            ui.spacing();
            ui.separator();
            ui.spacing();
            if world.resource::<GameModeSettings>().is_3d() {
                ui.spacing();
                if ui.button("Add 3D Playmode Camera").clicked() {
                    commands.spawn((
                        Camera3d::default(),
                        Name::new("Camera3d".to_string()),
                        Transform::default(),
                        Visibility::default(),
                        CameraPlay::default(),
                        PrefabMarker,
                    ));
                }
            } else if ui.button("Add 2D Playmode Camera").clicked() {
                commands.spawn((
                    Camera2d::default(),
                    Name::new("Camera2d".to_string()),
                    Transform::default(),
                    Visibility::default(),
                    CameraPlay::default(),
                    PrefabMarker,
                ));
            }
        }

        // Moves camera below the selection
        let pos = ui.next_widget_position();
        let mut clipped = ui.clip_rect();
        clipped.set_left(pos.x);
        clipped.set_top(pos.y);
        self.viewport_rect = Some(clipped);

        if self.target_image.is_none() {
            let handle = world
                .resource_mut::<Assets<Image>>()
                .add(create_camera_image(
                    clipped.width() as u32,
                    clipped.height() as u32,
                ));
            self.target_image = Some(handle);
            self.need_reinit_egui_tex = true;
        } else if let Some(handle) = &self.target_image {
            if let Some(image) = world.resource::<Assets<Image>>().get(handle) {
                if image.texture_descriptor.size.width != clipped.width() as u32
                    || image.texture_descriptor.size.height != clipped.height() as u32
                {
                    world
                        .resource_mut::<Assets<Image>>()
                        .get_mut(handle)
                        .unwrap()
                        .resize(Extent3d {
                            width: clipped.width() as u32,
                            height: clipped.height() as u32,
                            ..default()
                        });
                    self.need_reinit_egui_tex = true;
                }
            } else {
                self.target_image = None;
                self.need_reinit_egui_tex = true;
            }
        }

        if let Some((cam_image, _)) = self.egui_tex_id {
            ui.image(egui::load::SizedTexture {
                id: cam_image,
                size: ui.available_size(),
            });
        }
    }

    fn title(&self) -> bevy_egui_next::egui::WidgetText {
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

    info!("Clean camera view tab");
}

#[derive(Default)]
struct LastCamTabRect(Option<egui::Rect>);

fn set_camera_viewport(
    mut local: Local<LastCamTabRect>,
    mut ui_state: ResMut<CameraViewTab>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui_next::EguiSettings>,
    mut cameras: Query<(&mut Camera, &mut Transform), Without<EditorCameraMarker>>,
    mut ctxs: EguiContexts,
) {
    let Some(real_cam_entity) = ui_state.real_camera else {
        return;
    };

    let Some(camera_entity) = ui_state.camera_entity else {
        return;
    };

    let Some(target_image) = ui_state.target_image.clone() else {
        return;
    };

    if ui_state.egui_tex_id.is_none() {
        ui_state.egui_tex_id = Some((ctxs.add_image(target_image.clone()), target_image.clone()));
    }

    if ui_state.need_reinit_egui_tex {
        ctxs.remove_image(&ui_state.egui_tex_id.as_ref().unwrap().1);
        ui_state.egui_tex_id = Some((ctxs.add_image(target_image.clone()), target_image.clone()));
        ui_state.need_reinit_egui_tex = false;
    }

    let Ok([(mut real_cam, mut real_cam_transform), (watch_cam, camera_transform)]) =
        cameras.get_many_mut([real_cam_entity, camera_entity])
    else {
        if let Ok((mut real_cam, _)) = cameras.get_mut(real_cam_entity) {
            real_cam.is_active = false;
            ui_state.camera_entity = None;
        }
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
    // set editor params for real_cam
    real_cam.order = 2;
    real_cam.is_active = true;
    real_cam.target = RenderTarget::Image(target_image);

    *real_cam_transform = *camera_transform;

    #[cfg(target_os = "macos")]
    let mut scale_factor = window.scale_factor() * egui_settings.scale_factor;
    #[cfg(not(target_os = "macos"))]
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;
    let cam_aspect_ratio = watch_cam
        .logical_viewport_size()
        .map(|cam| cam.y as f64 / cam.x as f64);
    #[cfg(target_os = "macos")]
    if let Some(ratio) = cam_aspect_ratio {
        scale_factor *= ratio;
    }

    let mut viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let mut viewport_size = viewport_rect.size() * scale_factor as f32;

    // Fixes camera viewport size to be proportional to main watch camera
    if let Some(ratio) = cam_aspect_ratio {
        viewport_size.y = viewport_size.x * ratio as f32;
    }

    // Place viewport in the center of the tab
    viewport_pos.y += (viewport_rect.size().y - viewport_size.y) / 2.0;

    real_cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(0, (viewport_rect.size().y - viewport_size.y) as u32 / 2),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}
