use bevy::{prelude::*, render::camera::CameraProjection};
use bevy_egui::egui::{self, Key};
use space_editor_core::prelude::*;
use space_shared::*;
use transform_gizmo_bevy::{EnumSet, Gizmo, GizmoMode, GizmoTarget};

use crate::EditorGizmo;
use crate::{colors::*, sizing::Sizing};
use crate::{
    game_view::GameViewTab,
    icons::{rotation_icon, scale_icon, translate_icon},
    prelude::{CloneEvent, EditorTool},
    tool::ToolExt,
};

pub struct GizmoToolPlugin;

impl Plugin for GizmoToolPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        use transform_gizmo_bevy::GizmoOptions;

        app.editor_tool(GizmoTool::default());
        app.add_plugins(transform_gizmo_bevy::TransformGizmoPlugin);

        if let Some(mut game_view_tab) = app.world_mut().get_resource_mut::<GameViewTab>() {
            game_view_tab.active_tool = Some(0);
        }
        app.init_resource::<MultipleCenter>();

        app.editor_hotkey(GizmoHotkey::Translate, vec![KeyCode::KeyG]);
        app.editor_hotkey(GizmoHotkey::Rotate, vec![KeyCode::KeyR]);
        app.editor_hotkey(GizmoHotkey::Scale, vec![KeyCode::KeyS]);
        app.editor_hotkey(GizmoHotkey::Delete, vec![KeyCode::KeyX]);
        app.editor_hotkey(GizmoHotkey::Multiple, vec![KeyCode::ShiftLeft]);
        app.editor_hotkey(GizmoHotkey::Clone, vec![KeyCode::AltLeft]);

        app.add_systems(Update, draw_lines_system.in_set(EditorSet::Editor));

        app.add_systems(Update, toggle_picking_enabled.in_set(EditorSet::Editor));
        app.insert_resource(GizmoOptions {
            hotkeys: Some(transform_gizmo_bevy::GizmoHotkeys::default()),
            ..Default::default()
        });
    }
}

fn toggle_picking_enabled(
    gizmo_targets: Query<&GizmoTarget>,
    mut picking_settings: ResMut<PickingPlugin>,
) {
    // Picking is disabled when any of the gizmos is focused or active.

    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused() && !target.is_active());
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum GizmoHotkey {
    Translate,
    Rotate,
    Scale,
    Delete,
    Multiple,
    Clone,
}

impl Hotkey for GizmoHotkey {
    fn name<'a>(&self) -> String {
        match self {
            Self::Translate => "Translate entity".to_string(),
            Self::Rotate => "Rotate entity".to_string(),
            Self::Scale => "Scale entity".to_string(),
            Self::Delete => "Delete entity".to_string(),
            Self::Multiple => "Change multiple entities".to_string(),
            Self::Clone => "Clone entity".to_string(),
        }
    }
}

pub struct GizmoTool {
    pub gizmo_mode: EnumSet<GizmoMode>,
    pub is_move_cloned_entities: bool,
    pub gizmo: Gizmo,
}

impl Default for GizmoTool {
    fn default() -> Self {
        Self {
            gizmo_mode: GizmoMode::all_translate(),
            is_move_cloned_entities: false,
            gizmo: Gizmo::default(),
        }
    }
}

const MODE_TO_NAME: [(EnumSet<GizmoMode>, &str); 3] = [
    (GizmoMode::all_translate(), "Translate"),
    (GizmoMode::all_rotate(), "Rotate"),
    (GizmoMode::all_scale(), "Scale"),
];

//hot keys. Blender keys prefer
const MODE_TO_KEY: [(EnumSet<GizmoMode>, GizmoHotkey); 3] = [
    (GizmoMode::all_translate(), GizmoHotkey::Translate),
    (GizmoMode::all_rotate(), GizmoHotkey::Rotate),
    (GizmoMode::all_scale(), GizmoHotkey::Scale),
];

impl EditorTool for GizmoTool {
    fn name(&self) -> &str {
        "Gizmo"
    }

    fn ui(&mut self, ui: &mut egui::Ui, commands: &mut Commands, world: &mut World) {

        let sizing = world.resource::<Sizing>();

        ui.spacing();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            let stl = ui.style_mut();
            stl.spacing.button_padding = egui::Vec2::new(4., 2.);
            stl.spacing.item_spacing = egui::Vec2::new(1., 0.);
            for (mode, hint) in MODE_TO_NAME {
                if self.gizmo_mode == mode {
                    ui.add(mode.to_button(sizing).fill(SELECTED_ITEM_COLOR))
                        .on_disabled_hover_text(hint)
                        .on_hover_text(hint)
                        .clicked();
                } else if ui
                    .add(mode.to_button(sizing))
                    .on_disabled_hover_text(hint)
                    .on_hover_text(hint)
                    .clicked()
                {
                    self.gizmo_mode = mode;
                }
            }
        });

        let Some(input) = world.get_resource::<ButtonInput<GizmoHotkey>>() else {
            warn!("Failed to retrieve gizmos hotkey button input");
            return;
        };

        let mut del = false;
        let mut clone_pressed = false;
        let mut multiple_pressed = false;
        let mut disable_pan_orbit = false;

        for (mode, key) in MODE_TO_KEY {
            if input.just_pressed(key) {
                self.gizmo_mode = mode;
            }
        }

        if ui.input(|s| {
            input.just_pressed(GizmoHotkey::Delete)
                || (s.modifiers.shift && s.modifiers.ctrl && s.key_pressed(Key::Delete))
        }) {
            del = true;
        }

        if !input.pressed(GizmoHotkey::Clone) {
            self.is_move_cloned_entities = false;
        } else {
            clone_pressed = true;
        }

        if input.pressed(GizmoHotkey::Multiple) {
            multiple_pressed = true;
        }

        if del {
            let mut query = world.query_filtered::<Entity, With<Selected>>();
            for e in query.iter(world) {
                commands.entity(e).despawn_recursive();
            }
            return;
        }


        let selected;
        {
            let mut q_selected = world.query_filtered::<Entity, With<Selected>>();
            selected = q_selected.iter(&world).collect::<Vec<_>>();
        }

        let has_gizmo_targets;
        {
            let mut q_gizmo_targets = world.query_filtered::<Entity, With<GizmoTarget>>();
            has_gizmo_targets = q_gizmo_targets.iter(&world).collect::<Vec<_>>();
        }

        // Add gizmo targets to the selected without gizmo targets
        for s in selected.iter() {
            if !has_gizmo_targets.contains(s) {
                info!("Adding gizmo target to {:?}", s);
                commands.entity(*s).insert(GizmoTarget::default());
            }
        }
        // Remove gizmo targets from the gizmo which are not selected
        for s in has_gizmo_targets.iter() {
            if !selected.contains(s) {
                info!("Removing gizmo target from {:?}", s);
                commands.entity(*s).remove::<GizmoTarget>();
            }
        }


        if ui.ctx().wants_pointer_input() {
            disable_pan_orbit = true;
        }

        if disable_pan_orbit {
            unsafe {
                world.get_resource_mut::<crate::EditorCameraEnabled>()
                    .unwrap()
                    .0 = false
            };
        }
    }
}

#[derive(Resource, Default)]
pub struct MultipleCenter {
    pub center: Option<Vec3>,
}

trait ToButton {
    fn to_button(&self, size: &Sizing) -> egui::Button;
}

impl ToButton for EnumSet<GizmoMode> {
    fn to_button(&self, size: &Sizing) -> egui::Button {
        if *self == GizmoMode::all_translate() {
            return translate_icon(size.gizmos.to_size(), "");
        } else if *self == GizmoMode::all_scale() {
            return scale_icon(size.gizmos.to_size(), "");
        } else if *self == GizmoMode::all_rotate() {
            return rotation_icon(size.gizmos.to_size(), "");
        }

        return translate_icon(size.gizmos.to_size(), "");
    }
}

fn draw_lines_system(
    mut gizmos: Gizmos<EditorGizmo>,
    mean_center: Res<MultipleCenter>,
    selected: Query<&GlobalTransform, With<Selected>>,
) {
    if let Some(center) = mean_center.center {
        for selected in &selected {
            gizmos.line(selected.translation(), center, Color::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gizmo_tool() {
        let default_tool = GizmoTool::default();

        assert_eq!(default_tool.gizmo_mode, GizmoMode::all_translate());
        assert_eq!(default_tool.is_move_cloned_entities, false);
        assert_eq!(default_tool.name(), "Gizmo");
    }

    #[test]
    fn test_gizmo_hotkey_name() {
        let gizmo_hotkey = GizmoHotkey::Translate;
        assert_eq!(gizmo_hotkey.name(), "Translate entity");

        let gizmo_hotkey = GizmoHotkey::Rotate;
        assert_eq!(gizmo_hotkey.name(), "Rotate entity");

        let gizmo_hotkey = GizmoHotkey::Scale;
        assert_eq!(gizmo_hotkey.name(), "Scale entity");

        let gizmo_hotkey = GizmoHotkey::Delete;
        assert_eq!(gizmo_hotkey.name(), "Delete entity");

        let gizmo_hotkey = GizmoHotkey::Multiple;
        assert_eq!(gizmo_hotkey.name(), "Change multiple entities");

        let gizmo_hotkey = GizmoHotkey::Clone;
        assert_eq!(gizmo_hotkey.name(), "Clone entity");
    }
}
