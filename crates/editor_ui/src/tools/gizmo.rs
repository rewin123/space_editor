use bevy::{prelude::*, render::camera::CameraProjection};
use bevy_egui::egui::{self, Key};
use space_editor_core::prelude::*;
use space_shared::*;
use transform_gizmo_egui::{EnumSet, Gizmo, GizmoExt, GizmoMode};

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
        app.editor_tool(GizmoTool::default());

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
    }
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

fn bevy_to_gizmo_transform(transform: &Transform) -> transform_gizmo_egui::math::Transform {
    transform_gizmo_egui::math::Transform {
        scale: transform_gizmo_egui::mint::Vector3::<f64>::from_slice(
            &transform.scale.as_dvec3().to_array(),
        ),
        rotation: transform_gizmo_egui::mint::Quaternion::<f64> {
            v: transform_gizmo_egui::mint::Vector3::<f64> {
                x: transform.rotation.x as f64,
                y: transform.rotation.y as f64,
                z: transform.rotation.z as f64,
            },
            s: transform.rotation.w as f64,
        },
        translation: transform_gizmo_egui::mint::Vector3::<f64>::from_slice(
            &transform.translation.as_dvec3().to_array(),
        ),
    }
}

fn gizmo_to_bevy_transform(transform: &transform_gizmo_egui::math::Transform) -> Transform {
    Transform {
        scale: Vec3::new(
            transform.scale.x as f32,
            transform.scale.y as f32,
            transform.scale.z as f32,
        ),
        translation: Vec3::new(
            transform.translation.x as f32,
            transform.translation.y as f32,
            transform.translation.z as f32,
        ),
        rotation: Quat::from_xyzw(
            transform.rotation.v.x as f32,
            transform.rotation.v.y as f32,
            transform.rotation.v.z as f32,
            transform.rotation.s as f32,
        ),
        ..Default::default()
    }
}

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

        let (cam_transform, cam_proj) = {
            let mut cam_query =
                world.query_filtered::<(&GlobalTransform, &Projection), With<EditorCameraMarker>>();
            let Ok((ref_tr, ref_cam)) = cam_query.get_single(world) else {
                return;
            };
            (*ref_tr, ref_cam.clone())
        };

        let selected = world
            .query_filtered::<Entity, With<Selected>>()
            .iter(world)
            .collect::<Vec<_>>();
        let mut disable_pan_orbit = false;

        let cell = world.as_unsafe_world_cell();

        let view_matrix = Mat4::from(cam_transform.affine().inverse());
        if multiple_pressed {
            let mut mean_transform = Transform::IDENTITY;
            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(global_transform) = (unsafe { ecell.get::<GlobalTransform>() }) else {
                    continue;
                };
                let tr = global_transform.compute_transform();
                mean_transform.translation += tr.translation;
                mean_transform.scale += tr.scale;
            }
            mean_transform.translation /= selected.len() as f32;
            mean_transform.scale /= selected.len() as f32;

            let mut global_mean = GlobalTransform::from(mean_transform);

            unsafe {
                cell.world_mut().insert_resource(MultipleCenter {
                    center: Some(global_mean.translation()),
                });
            }

            let mut loc_transform = vec![];
            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(global_transform) = (unsafe { ecell.get::<GlobalTransform>() }) else {
                    continue;
                };
                loc_transform.push(global_transform.reparented_to(&global_mean));
            }

            let mut gizmo_interacted = false;

            let proj_mat = cam_proj.get_clip_from_view();
            let proj_mat = transform_gizmo_egui::math::DMat4 {
                x_axis: transform_gizmo_egui::math::DVec4::from_array(
                    proj_mat.x_axis.as_dvec4().to_array(),
                ),
                y_axis: transform_gizmo_egui::math::DVec4::from_array(
                    proj_mat.y_axis.as_dvec4().to_array(),
                ),
                z_axis: transform_gizmo_egui::math::DVec4::from_array(
                    proj_mat.z_axis.as_dvec4().to_array(),
                ),
                w_axis: transform_gizmo_egui::math::DVec4::from_array(
                    proj_mat.w_axis.as_dvec4().to_array(),
                ),
            };

            let view_matrix = transform_gizmo_egui::math::DMat4 {
                x_axis: transform_gizmo_egui::math::DVec4::from_array(
                    view_matrix.x_axis.as_dvec4().to_array(),
                ),
                y_axis: transform_gizmo_egui::math::DVec4::from_array(
                    view_matrix.y_axis.as_dvec4().to_array(),
                ),
                z_axis: transform_gizmo_egui::math::DVec4::from_array(
                    view_matrix.z_axis.as_dvec4().to_array(),
                ),
                w_axis: transform_gizmo_egui::math::DVec4::from_array(
                    view_matrix.w_axis.as_dvec4().to_array(),
                ),
            };

            let gizmo_config = transform_gizmo_egui::GizmoConfig {
                projection_matrix: proj_mat.into(),
                view_matrix: view_matrix.into(),
                modes: self.gizmo_mode.into(),
                viewport: ui.clip_rect(),
                ..Default::default()
            };

            self.gizmo.update_config(gizmo_config);

            info!("{:?}", &mean_transform);

            if let Some((_, transforms)) = self
                .gizmo
                .interact(ui, &[bevy_to_gizmo_transform(&mean_transform)])
            {
                gizmo_interacted = true;
                mean_transform = gizmo_to_bevy_transform(&transforms[0]);
                disable_pan_orbit = true;
            }

            global_mean = GlobalTransform::from(mean_transform);

            if gizmo_interacted && clone_pressed {
                if self.is_move_cloned_entities {
                } else {
                    for e in selected.iter() {
                        unsafe { cell.world_mut().send_event(CloneEvent { id: *e }) };
                    }
                    self.is_move_cloned_entities = true;
                    return;
                }
            }

            if gizmo_interacted {
                for (idx, e) in selected.iter().enumerate() {
                    let Some(ecell) = cell.get_entity(*e) else {
                        continue;
                    };
                    let Some(mut transform) = (unsafe { ecell.get_mut::<Transform>() }) else {
                        continue;
                    };

                    let new_global = global_mean.mul_transform(loc_transform[idx]);

                    if let Some(parent) = unsafe { ecell.get::<Parent>() } {
                        if let Some(parent) = cell.get_entity(parent.get()) {
                            if let Some(parent_global) = unsafe { parent.get::<GlobalTransform>() }
                            {
                                *transform = new_global.reparented_to(parent_global);
                            }
                        }
                    } else {
                        *transform = new_global.compute_transform();
                    }
                }
            }
        } else {
            unsafe {
                cell.world_mut()
                    .insert_resource(MultipleCenter { center: None });
            }

            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut transform) = (unsafe { ecell.get_mut::<Transform>() }) else {
                    continue;
                };

                let proj_mat = cam_proj.get_clip_from_view();
                let proj_mat = transform_gizmo_egui::math::DMat4 {
                    x_axis: transform_gizmo_egui::math::DVec4::from_array(
                        proj_mat.x_axis.as_dvec4().to_array(),
                    ),
                    y_axis: transform_gizmo_egui::math::DVec4::from_array(
                        proj_mat.y_axis.as_dvec4().to_array(),
                    ),
                    z_axis: transform_gizmo_egui::math::DVec4::from_array(
                        proj_mat.z_axis.as_dvec4().to_array(),
                    ),
                    w_axis: transform_gizmo_egui::math::DVec4::from_array(
                        proj_mat.w_axis.as_dvec4().to_array(),
                    ),
                };

                let view_matrix = transform_gizmo_egui::math::DMat4 {
                    x_axis: transform_gizmo_egui::math::DVec4::from_array(
                        view_matrix.x_axis.as_dvec4().to_array(),
                    ),
                    y_axis: transform_gizmo_egui::math::DVec4::from_array(
                        view_matrix.y_axis.as_dvec4().to_array(),
                    ),
                    z_axis: transform_gizmo_egui::math::DVec4::from_array(
                        view_matrix.z_axis.as_dvec4().to_array(),
                    ),
                    w_axis: transform_gizmo_egui::math::DVec4::from_array(
                        view_matrix.w_axis.as_dvec4().to_array(),
                    ),
                };

                let gizmo_config = transform_gizmo_egui::GizmoConfig {
                    projection_matrix: proj_mat.into(),
                    view_matrix: view_matrix.into(),
                    modes: self.gizmo_mode.into(),
                    viewport: ui.clip_rect(),
                    ..Default::default()
                };

                if let Some(parent) = unsafe { ecell.get::<Parent>() } {
                    if let Some(parent) = cell.get_entity(parent.get()) {
                        if let Some(parent_global) = unsafe { parent.get::<GlobalTransform>() } {
                            if let Some(global) = unsafe { ecell.get::<GlobalTransform>() } {
                                self.gizmo.update_config(gizmo_config);

                                if let Some((_, transforms)) = self.gizmo.interact(
                                    ui,
                                    &[bevy_to_gizmo_transform(&global.compute_transform())],
                                ) {
                                    disable_pan_orbit = true;
                                    let new_transform = gizmo_to_bevy_transform(&transforms[0]);

                                    if clone_pressed {
                                        if self.is_move_cloned_entities {
                                            let new_transform =
                                                GlobalTransform::from(new_transform);
                                            *transform = new_transform.reparented_to(parent_global);
                                            transform.set_changed();
                                            disable_pan_orbit = true;
                                        } else {
                                            unsafe {
                                                cell.world_mut().send_event(CloneEvent { id: *e })
                                            };
                                            self.is_move_cloned_entities = true;
                                        }
                                    } else {
                                        let new_transform = GlobalTransform::from(new_transform);
                                        *transform = new_transform.reparented_to(parent_global);
                                        transform.set_changed();
                                    }
                                }
                                continue;
                            }
                        }
                    }
                }

                self.gizmo.update_config(gizmo_config);

                if let Some((_, transforms)) = self
                    .gizmo
                    .interact(ui, &[bevy_to_gizmo_transform(&transform)])
                {
                    if clone_pressed {
                        if self.is_move_cloned_entities {
                            *transform = gizmo_to_bevy_transform(&transforms[0]);
                            transform.set_changed();
                        } else {
                            unsafe { cell.world_mut().send_event(CloneEvent { id: *e }) };
                            self.is_move_cloned_entities = true;
                        }
                    } else {
                        *transform = gizmo_to_bevy_transform(&transforms[0]);
                        transform.set_changed();
                    }
                    disable_pan_orbit = true;
                }
            }
        }

        if ui.ctx().wants_pointer_input() {
            disable_pan_orbit = true;
        }

        if disable_pan_orbit {
            unsafe {
                cell.get_resource_mut::<crate::EditorCameraEnabled>()
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
