
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_egui::egui;

use crate::{prelude::EditorTab, EditorCameraMarker};

#[derive(Default, Resource)]
pub struct GameViewTab {
    pub viewport_rect : Option<egui::Rect>
}

impl EditorTab for GameViewTab {
    fn ui(&mut self, ui : &mut bevy_egui::egui::Ui, world : &mut World) {
        self.viewport_rect = Some(ui.clip_rect());

        // GIZMO DRAW
        // Draw gizmo per entity to individual move
        // If SHIFT pressed draw "mean" gizmo to move all selected entities together

        let cam_query = world.query_filtered::<(&Transform, &Camera), With<EditorCameraMarker>>();
        let (cam_transform, camera) = {
            let (ref_tr, ref_cam) = cam_query.single(world);
            (ref_tr.clone(), ref_cam.clone()) 
        };

        let view_matrix = Mat4::from(cam_transform.affine().inverse());
        if ui.input(|s| s.modifiers.shift) {
            let mut mean_transform = Transform::IDENTITY;
            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut global_transform) = ecell.get_mut::<GlobalTransform>() else {
                    continue;
                };
                let tr = global_transform.compute_transform();
                mean_transform.translation += tr.translation;
                mean_transform.scale += tr.scale;
            }
            mean_transform.translation /= selected.len() as f32;
            mean_transform.scale /= selected.len() as f32;

            let mut global_mean = GlobalTransform::from(mean_transform);

            let mut loc_transform = vec![];
            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut global_transform) = ecell.get_mut::<GlobalTransform>() else {
                    continue;
                };
                loc_transform.push(global_transform.reparented_to(&global_mean));
            }

            if let Some(result) = egui_gizmo::Gizmo::new(format!("Selected gizmo mean global"))
                .projection_matrix(cam_proj.get_projection_matrix().to_cols_array_2d())
                .view_matrix(view_matrix.to_cols_array_2d())
                .model_matrix(mean_transform.compute_matrix().to_cols_array_2d())
                .mode(state.gizmo_mode.clone())
                .interact(ui) {

                mean_transform = Transform {
                    translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                    rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                    scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                };
                disable_pan_orbit = true;
            }

            global_mean = GlobalTransform::from(mean_transform);

            for (idx, e) in selected.iter().enumerate() {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut transform) = ecell.get_mut::<Transform>() else {
                    continue;
                };

                let new_global = global_mean.mul_transform(loc_transform[idx]);

                if let Some(parent) = ecell.get::<Parent>() {
                    if let Some(parent) = cell.get_entity(parent.get()) {
                        if let Some(parent_global) = parent.get::<GlobalTransform>() {
                            *transform = new_global.reparented_to(&parent_global);
                        }
                    }
                } else {
                    *transform = new_global.compute_transform();
                }
            }
        } else {
            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut transform) = ecell.get_mut::<Transform>() else {
                    continue;
                };
                if let Some(parent) = ecell.get::<Parent>() {
                    if let Some(parent) = cell.get_entity(parent.get()) {
                        if let Some(parent_global) = parent.get::<GlobalTransform>() {
                            if let Some(global) = ecell.get::<GlobalTransform>() {
                                if let Some(result) = egui_gizmo::Gizmo::new(format!("Selected gizmo {:?}", *e))
                                    .projection_matrix(cam_proj.get_projection_matrix().to_cols_array_2d())
                                    .view_matrix(view_matrix.to_cols_array_2d())
                                    .model_matrix(global.compute_matrix().to_cols_array_2d())
                                    .mode(state.gizmo_mode.clone())
                                    .interact(ui) {
                                    
                                    let new_transform = Transform {
                                        translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                                        rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                                        scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                                    };

                                    let new_transform = GlobalTransform::from(new_transform);
                                    *transform = new_transform.reparented_to(&parent_global);
                                    transform.set_changed();
                                }
                                disable_pan_orbit = true;
                                continue;
                            }
                        }
                    }
                }
                if let Some(result) = egui_gizmo::Gizmo::new(format!("Selected gizmo {:?}", *e))
                    .projection_matrix(cam_proj.get_projection_matrix().to_cols_array_2d())
                    .view_matrix(view_matrix.to_cols_array_2d())
                    .model_matrix(transform.compute_matrix().to_cols_array_2d())
                    .mode(state.gizmo_mode.clone())
                    .interact(ui) {

                    *transform = Transform {
                        translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                        rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                        scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                    };
                    transform.set_changed();
                    disable_pan_orbit = true;
                }
            }
        }
    }

    fn title(&self) -> bevy_egui::egui::WidgetText {
        "Game view".into()
    }
}