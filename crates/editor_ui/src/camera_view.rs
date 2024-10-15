use bevy::{
    core_pipeline::tonemapping::DebandDither,
    prelude::*,
    render::{
        camera::{CameraRenderGraph, RenderTarget, TemporalJitter},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, RichText},
    EguiContexts,
};

use space_prefab::component::PlaymodeCamera;
use space_shared::{toast::ToastMessage, *};

use crate::{
    editor_tab_name::EditorTabName, prelude::GameModeSettings, DisableCameraSkip, RenderLayers,
};

use space_editor_tabs::prelude::*;

use crate::colors::*;

pub struct CameraViewTabPlugin;

impl Plugin for CameraViewTabPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(CameraViewTab::default());
        app.add_systems(PreUpdate, set_camera_viewport.in_set(EditorSet::Editor));
        app.add_systems(OnEnter(EditorState::Game), clean_camera_view_tab);
    }
}

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
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, commands: &mut Commands, world: &mut World) {
        if self.real_camera.is_none() {
            if world
                .get_resource::<GameModeSettings>()
                .map_or(false, |mode| mode.is_3d())
            {
                self.real_camera = Some(
                    commands
                        .spawn((
                            Camera3dBundle {
                                camera: Camera {
                                    is_active: true,
                                    order: 2,
                                    clear_color: bevy::render::camera::ClearColorConfig::Default,
                                    ..default()
                                },
                                ..default()
                            },
                            RenderLayers::layer(0),
                            TemporalJitter::default(),
                            Name::new("Camera for Camera view tab"),
                            DisableCameraSkip,
                        ))
                        .id(),
                );
            } else if world
                .get_resource::<GameModeSettings>()
                .map_or(false, |mode| mode.is_2d())
            {
                self.real_camera = Some(
                    commands
                        .spawn((
                            Camera2dBundle {
                                camera: Camera {
                                    is_active: false,
                                    order: 2,
                                    clear_color: bevy::render::camera::ClearColorConfig::Default,
                                    ..default()
                                },
                                ..default()
                            },
                            RenderLayers::layer(0),
                            Name::new("Camera for Camera view tab"),
                            DisableCameraSkip,
                        ))
                        .id(),
                );
            }
        }

        let mut camera_query = world.query_filtered::<Entity, (
            With<Camera>,
            With<PlaymodeCamera>,
            Without<EditorCameraMarker>,
        )>();

        if camera_query.iter(world).count() == 1 {
            let selected_entity = camera_query.iter(world).next();
            self.camera_entity = selected_entity;

            if let Some(entity) = selected_entity {
                ui.label(format!("Camera: {:?}", entity));
            } else {
                ui.label(RichText::new("No selected Camera").color(ERROR_COLOR));
            }
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
            if world
                .get_resource::<GameModeSettings>()
                .map_or(false, |mode| mode.is_3d())
            {
                ui.spacing();
                if ui.button("Add 3D Playmode Camera").clicked() {
                    commands.spawn((
                        Camera3d::default(),
                        Camera::default(),
                        DebandDither::Enabled,
                        Projection::Perspective(PerspectiveProjection::default()),
                        Name::new("Camera3d".to_string()),
                        Transform::default(),
                        VisibilityBundle::default(),
                        PlaymodeCamera::default(),
                        PrefabMarker,
                        CameraRenderGraph::new(bevy::core_pipeline::core_3d::graph::Core3d),
                    ));
                }
            } else if ui.button("Add 2D Playmode Camera").clicked() {
                commands.spawn((
                    Camera2d {},
                    Name::new("Camera2d".to_string()),
                    Transform::default(),
                    VisibilityBundle::default(),
                    PlaymodeCamera::default(),
                    CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::Core2d),
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

        let mut need_recreate_texture = false;

        if self.target_image.is_none() {
            let Some(handle) = world.get_resource_mut::<Assets<Image>>().map(|mut assets| {
                assets.add(create_camera_image(
                    clipped.width() as u32,
                    clipped.height() as u32,
                ))
            }) else {
                world.send_event(ToastMessage::new(
                    "No camera image target found.",
                    toast::ToastKind::Error,
                ));
                return;
            };
            self.target_image = Some(handle);
            self.need_reinit_egui_tex = true;
            let msg = format!(
                "Camera target created. W: {}, H: {}",
                clipped.width(),
                clipped.height()
            );
            world.send_event(ToastMessage::new(&msg, toast::ToastKind::Success));
        } else if let Some(handle) = &self.target_image {
            if let Some(image) = world
                .get_resource::<Assets<Image>>()
                .and_then(|asset| asset.get(handle))
            {
                if image.texture_descriptor.size.width != clipped.width() as u32
                    || image.texture_descriptor.size.height != clipped.height() as u32
                {
                    need_recreate_texture = true;
                    self.need_reinit_egui_tex = true;
                }
            } else {
                self.target_image = None;
                self.need_reinit_egui_tex = true;
            }
        }

        if need_recreate_texture {
            let Some(handle) = world.get_resource_mut::<Assets<Image>>().map(|mut assets| {
                assets.add(create_camera_image(
                    clipped.width() as u32,
                    clipped.height() as u32,
                ))
            }) else {
                world.send_event(ToastMessage::new(
                    "No camera image target found.",
                    toast::ToastKind::Error,
                ));
                return;
            };

            self.target_image = Some(handle);
            self.need_reinit_egui_tex = true;
        }

        if let Some((cam_image, _)) = self.egui_tex_id {
            ui.image(egui::load::SizedTexture {
                id: cam_image,
                size: ui.available_size(),
            });
        }
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::CameraView.into()
    }
}

fn clean_camera_view_tab(
    mut ui_state: ResMut<CameraViewTab>,
    mut cameras: Query<(&mut Camera, &mut GlobalTransform), Without<EditorCameraMarker>>,
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

    info!("Clean camera view tab successful");
}

#[derive(Default)]
struct LastCamTabRect(Option<egui::Rect>);

fn set_camera_viewport(
    mut local: Local<LastCamTabRect>,
    mut ui_state: ResMut<CameraViewTab>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<
        (&mut Camera, &mut GlobalTransform, &mut Transform),
        Without<EditorCameraMarker>,
    >,
    mut ctxs: EguiContexts,
    images: Res<Assets<Image>>,
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
        ui_state.target_image = Some(target_image.clone());
        ui_state.egui_tex_id = Some((ctxs.add_image(target_image.clone()), target_image.clone()));
    }

    if let (Some((_tx_id, handle)), true) = (&ui_state.egui_tex_id, ui_state.need_reinit_egui_tex) {
        ctxs.remove_image(handle);
        ui_state.egui_tex_id = Some((ctxs.add_image(target_image.clone()), target_image));
        ui_state.need_reinit_egui_tex = false;
    }

    let Ok([(mut real_cam, _, mut real_cam_local_transform), (watch_cam, camera_transform, _)]) =
        cameras.get_many_mut([real_cam_entity, camera_entity])
    else {
        if let Ok((mut real_cam, _, _)) = cameras.get_mut(real_cam_entity) {
            real_cam.is_active = false;
            ui_state.camera_entity = None;
        }
        return;
    };

    let Ok(_) = primary_window.get_single() else {
        return;
    };

    let Some(viewport_rect) = ui_state.viewport_rect else {
        local.0 = None;
        warn!("No viewport rect for UI");
        return;
    };

    if watch_cam.is_changed() {
        *real_cam = watch_cam.clone();
    }
    // set editor params for real_cam
    real_cam.is_active = true;
    let Some(target_handle) = ui_state.target_image.clone() else {
        return;
    };
    real_cam.target = RenderTarget::Image(target_handle.clone());

    *real_cam_local_transform = camera_transform.compute_transform();

    local.0 = Some(viewport_rect);

    let Some(image_data) = images.get(target_handle.id()) else {
        error!("Could not get image data");
        return;
    };

    let image_rect = Rect::new(
        0.0,
        0.0,
        image_data.texture_descriptor.size.width as f32 - 10.0,
        image_data.texture_descriptor.size.height as f32 - 10.0,
    );

    let cam_aspect_ratio = watch_cam
        .logical_viewport_size()
        .map(|cam| cam.y as f64 / cam.x as f64);

    let mut preferred_height = image_rect.height();
    let mut preferred_width = image_rect.width();

    // Fixes camera viewport size to be proportional to main watch camera
    if let Some(ratio) = cam_aspect_ratio {
        preferred_height = image_rect.size().x * ratio as f32;
    }

    preferred_width = preferred_width.min(image_rect.size().x);
    preferred_height = preferred_height.min(image_rect.size().y);

    let view_image_rect = Rect::from_center_half_size(
        Vec2::new(image_rect.center().x, image_rect.center().y),
        Vec2::new(preferred_width, preferred_height) / 2.0,
    );

    let new_viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(view_image_rect.min.x as u32, view_image_rect.min.y as u32),
        physical_size: UVec2::new(
            view_image_rect.size().x as u32,
            view_image_rect.size().y as u32,
        ),
        depth: 0.0..1.0,
    });

    real_cam.viewport = new_viewport;
}
