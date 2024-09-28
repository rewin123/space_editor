use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::egui::{self, RichText, Widget};
use space_undo::UndoRedo;
use transform_gizmo_egui::GizmoMode;

use space_shared::*;

use crate::editor_tab_name::EditorTabName;

use super::tool::EditorTool;
use space_editor_tabs::prelude::*;

use crate::colors::*;

pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(GameViewTab::default());
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
            gizmo_mode: GizmoMode::TranslateView,
            smoothed_dt: 0.0,
            tools: vec![],
            active_tool: None,
        }
    }
}

impl EditorTab for GameViewTab {
    fn ui(&mut self, ui: &mut bevy_egui::egui::Ui, commands: &mut Commands, world: &mut World) {
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
            if let Some(dt) = world.get_resource::<Time>() {
                let dt = dt.delta_seconds();
                self.smoothed_dt = self.smoothed_dt.mul_add(0.98, dt * 0.02);
                ui.colored_label(TEXT_COLOR, format!("FPS: {:.0}", 1.0 / self.smoothed_dt));
            }

            #[cfg(debug_assertions)]
            {
                // spacing = available_width - button_widt - margin
                let button_distance = ui.available_width() - 92.0 - 8.0;
                ui.add_space(button_distance);
                warn_if_debug_build(ui);
            }
        });
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::GameView.into()
    }
}

pub fn warn_if_debug_build(ui: &mut egui::Ui) {
    if cfg!(debug_assertions) {
        egui::Button::new(RichText::new("⚠ Debug build").color(SPECIAL_BG_COLOR))
            .fill(WARN_COLOR)
            .ui(ui)
            .on_hover_text("space_editor was compiled with debug assertions enabled.");
    }
}

pub fn reset_camera_viewport(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<EditorCameraMarker>>,
    mut game_view_tab: ResMut<GameViewTab>,
) {
    let Ok(mut cam) = cameras.get_single_mut() else {
        return;
    };

    let Ok(_window) = primary_window.get_single() else {
        return;
    };

    game_view_tab.viewport_rect = None;

    cam.viewport = None;
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
    egui_settings: Res<bevy_egui::EguiSettings>,
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

    let scale_factor = window.scale_factor();
    debug!(
        "Window scale factor: {} egui scale factor: {}",
        scale_factor, egui_settings.scale_factor
    );

    let mut viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor;
    let mut viewport_size = viewport_rect.size() * scale_factor;

    viewport_pos.x = viewport_pos.x.max(0.0);
    viewport_pos.y = viewport_pos.y.max(0.0);

    viewport_size.x = viewport_size
        .x
        .min(window.width().mul_add(scale_factor, -viewport_pos.x));
    viewport_size.y = viewport_size
        .y
        .min(window.height().mul_add(scale_factor, -viewport_pos.y));

    if (viewport_size.x <= 0.0) || (viewport_size.y <= 0.0) {
        return;
    }
    cam.viewport = Some(bevy::render::camera::Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_game_view_tab() {
        let default_tab = GameViewTab::default();

        assert_eq!(default_tab.viewport_rect, None);
        assert_eq!(default_tab.gizmo_mode, GizmoMode::TranslateView);
        assert_eq!(default_tab.smoothed_dt, 0.0);
        assert_eq!(default_tab.tools.len(), 0);
        assert_eq!(default_tab.active_tool, None);
    }
}
