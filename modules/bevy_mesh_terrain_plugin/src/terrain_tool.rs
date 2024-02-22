use std::{
    any::Any,
    fmt::{self, Formatter},
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;
use bevy_mesh_terrain::edit::BrushType;
use space_editor_ui::prelude::*;

pub struct TerrainToolPlugin;

impl Plugin for TerrainToolPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tool(TerrainTools::default());
    }
}

#[derive(Default, Resource, Clone)]
pub struct TerrainTools {
    pub tool_mode: ToolMode,
    pub brush_type: BrushType,
    pub brush_radius: u32,
    pub brush_hardness: u32,
    pub color: LinearPixelColor, //brush mode
}

impl EditorTool for TerrainTools {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::bevy_egui_next::egui::Ui,
        _commands: &mut Commands,
        _world: &mut World,
    ) {
        ui.vertical(|ui| {
            ui.heading("Tool Mode");
            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.spacing();
                egui::ComboBox::new("tool_mode", "")
                    .selected_text(self.tool_mode.to_string())
                    .show_ui(ui, |ui| {
                        for tool_mode in TOOL_MODES.into_iter() {
                            if ui
                                .selectable_label(
                                    self.tool_mode == tool_mode,
                                    tool_mode.to_string(),
                                )
                                .clicked()
                            {
                                self.tool_mode = tool_mode;
                            }
                        }
                    });
            });
            ui.spacing();
            ui.separator();

            ui.add(egui::Slider::new(&mut self.brush_radius, 0..=100).text("Brush Radius"));
            ui.add(egui::Slider::new(&mut self.brush_hardness, 0..=100).text("Brush Hardness"));

            match self.tool_mode {
                ToolMode::Splat => {
                    ui.add(
                        egui::Slider::new(&mut self.color.r, 0..=255).text("Texture A (R_Channel"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.color.g, 0..=255).text("Texture B (G_Channel"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.color.b, 0..=255).text("Layer Fade (B_Channel"),
                    );
                }
                ToolMode::Height => {
                    egui::ComboBox::new("brush_type", "")
                        .selected_text(self.brush_type.to_string())
                        .show_ui(ui, |ui| {
                            for brush_type in BRUSH_TYPES.into_iter() {
                                if ui
                                    .selectable_label(
                                        self.brush_type == brush_type,
                                        brush_type.to_string(),
                                    )
                                    .clicked()
                                {
                                    self.brush_type = brush_type;
                                }
                            }
                        });

                    ui.add(
                        egui::Slider::new(&mut self.color.r, 0..=65535).text("Height (R_Channel)"),
                    );
                }
            }
        });
    }

    fn name(&self) -> &str {
        "Terrain Tools"
    }
}

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub enum ToolMode {
    #[default]
    Height,
    Splat,
}
const TOOL_MODES: [ToolMode; 2] = [ToolMode::Height, ToolMode::Splat];

const BRUSH_TYPES: [BrushType; 3] = [BrushType::SetExact, BrushType::Smooth, BrushType::Noise];

impl std::fmt::Display for ToolMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            ToolMode::Height => "Height",
            ToolMode::Splat => "Splat",
        };

        write!(f, "{}", label)
    }
}

#[derive(Default, Resource, Clone)]
pub struct LinearPixelColor {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}
