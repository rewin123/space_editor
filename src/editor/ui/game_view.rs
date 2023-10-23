use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::egui::{self};
use egui_gizmo::GizmoMode;

use crate::{editor::core::EditorTool, prelude::EditorTab, EditorCameraMarker};

#[derive(Resource)]
pub struct GameViewTab {
    pub viewport_rect: Option<egui::Rect>,
    pub tools: Vec<Box<dyn EditorTool + 'static + Send + Sync>>,
    pub active_tool: Option<usize>,
    pub gizmo_mode: GizmoMode,
    pub smoothed_dt: f32,
}

impl Default for GameViewTab {
    fn default() -> Self {
        Self {
            viewport_rect: None,
            gizmo_mode: GizmoMode::Translate,
            smoothed_dt: 0.0,
            tools: vec![],
            active_tool: None,
        }
    }
}

impl EditorTab for GameViewTab {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, world: &mut World) {
        self.viewport_rect = Some(ui.clip_rect());

        //Draw FPS
        let dt = world.get_resource::<Time>().unwrap().delta_seconds();
        self.smoothed_dt = self.smoothed_dt * 0.98 + dt * 0.02;
        ui.colored_label(
            egui::Color32::WHITE,
            format!("FPS: {:.0}", 1.0 / self.smoothed_dt),
        );

        //Tool processing
        if self.tools.is_empty() {
            return;
        }

        let selected_tool_name = if let Some(tool_id) = self.active_tool {
            self.tools[tool_id].name()
        } else {
            "None"
        };

        ui.style_mut().visuals.override_text_color = Some(egui::Color32::WHITE);
        egui::ComboBox::new("tool", "Tool")
            .selected_text(selected_tool_name)
            .show_ui(ui, |ui| {
                for (i, tool) in self.tools.iter().enumerate() {
                    if ui
                        .selectable_label(self.active_tool == Some(i), tool.name())
                        .clicked()
                    {
                        self.active_tool = Some(i);
                    }
                }
            });

        if let Some(tool_id) = self.active_tool {
            self.tools[tool_id].ui(ui, world);
        }
    }

    fn title(&self) -> bevy_egui::egui::WidgetText {
        "Game view".into()
    }
}

pub fn reset_camera_viewport(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(window.width() as u32, window.height() as u32),
        depth: 0.0..1.0,
    });
}

pub fn set_camera_viewport(
    ui_state: Res<GameViewTab>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(viewport_rect) = ui_state.viewport_rect else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}
