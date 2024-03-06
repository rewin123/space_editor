use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui_next::egui::{self};
use egui_gizmo::GizmoMode;
use space_undo::UndoRedo;

use space_shared::*;

use crate::{colors::TEXT_COLOR, prelude::EditorTabName, EditorUiAppExt};

use super::{editor_tab::EditorTab, tool::EditorTool};

pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());
    }
}

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
    fn ui(
        &mut self,
        ui: &mut bevy_egui_next::egui::Ui,
        commands: &mut Commands,
        world: &mut World,
    ) {
        if ui.input_mut(|i| i.key_released(egui::Key::Z) && i.modifiers.ctrl && !i.modifiers.shift)
        {
            world.send_event(UndoRedo::Undo);
            info!("Undo command");
        }
        if ui.input_mut(|i| i.key_released(egui::Key::Z) && i.modifiers.ctrl && i.modifiers.shift) {
            world.send_event(UndoRedo::Redo);
            info!("Redo command");
        }

        self.viewport_rect = Some(ui.clip_rect());

        ui.horizontal(|ui| {
            ui.style_mut().visuals.override_text_color = Some(TEXT_COLOR);

            //Tool processing
            if self.tools.is_empty() {
                return;
            }

            let selected_tool_name = if let Some(tool_id) = self.active_tool {
                self.tools[tool_id].name()
            } else {
                "None"
            };

            if self.tools.len() > 1 {
                egui::ComboBox::new("tool", "")
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
            }

            if let Some(tool_id) = self.active_tool {
                self.tools[tool_id].ui(ui, commands, world);
            }

            ui.spacing();
            //Draw FPS
            let dt = world.get_resource::<Time>().unwrap().delta_seconds();
            self.smoothed_dt = self.smoothed_dt.mul_add(0.98, dt * 0.02);
            ui.colored_label(TEXT_COLOR, format!("FPS: {:.0}", 1.0 / self.smoothed_dt));
        });
    }

    fn title(&self) -> bevy_egui_next::egui::WidgetText {
        "Game view".into()
    }
}

pub fn reset_camera_viewport(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
    mut game_view_tab: ResMut<GameViewTab>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    game_view_tab.viewport_rect = None;

    cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(window.width() as u32, window.height() as u32),
        depth: 0.0..1.0,
    });
}

pub fn has_window_changed(mut events: EventReader<bevy::window::WindowResized>) -> bool {
    events.read().next().is_some()
}

#[derive(Default)]
pub struct LastGameTabRect(Option<egui::Rect>);

pub fn set_camera_viewport(
    mut local: Local<LastGameTabRect>,
    ui_state: Res<GameViewTab>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui_next::EguiSettings>,
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
) {
    let Ok(mut cam) = cameras.get_single_mut() else {
        return;
    };

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let Some(viewport_rect) = ui_state.viewport_rect else {
        local.0 = None;
        return;
    };

    if local.0 == Some(viewport_rect) {
        return;
    }
    local.0 = Some(viewport_rect);

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}
