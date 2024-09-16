use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use space_shared::toast::*;

#[derive(Default)]
pub struct MouseCheck;

impl Plugin for MouseCheck {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerContextCheck>()
            .add_systems(Startup, initialize_mouse_context)
            .add_systems(PreUpdate, update_mouse_context);
    }
}

#[derive(Resource)]
pub struct PointerContextCheck {
    pointer_is_valid: bool,
    primary_window: Option<Entity>,
}

impl Default for PointerContextCheck {
    fn default() -> Self {
        Self {
            pointer_is_valid: true,
            primary_window: None,
        }
    }
}

pub fn initialize_mouse_context(
    mut toast: EventWriter<ToastMessage>,
    mut pointer_ctx: ResMut<PointerContextCheck>,
    window_q: Query<Entity, With<PrimaryWindow>>,
) {
    if let Ok(window_id) = window_q.get_single() {
        pointer_ctx.primary_window = Some(window_id);
    } else {
        toast.send(ToastMessage::new(
            "Could not get Primary Window",
            ToastKind::Error,
        ));
        error!("could not get Primary Window");
    }
}

pub fn update_mouse_context(
    mut pointer_ctx: ResMut<PointerContextCheck>,
    mut egui_ctxs: EguiContexts,
) {
    if let Some(window_id) = pointer_ctx.primary_window {
        pointer_ctx.pointer_is_valid = !egui_ctxs
            .ctx_for_entity_mut(window_id)
            .wants_pointer_input();
    }
}
