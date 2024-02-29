use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui_next::EguiContext;
use egui_dock::egui::{self, Align2};

pub use egui_toast::*;
pub struct ToastUiPlugin;

impl Plugin for ToastUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastStorage>()
            .add_event::<ToastMessage>()
            .add_systems(Update, read_toast)
            .add_systems(PostUpdate, show_toast);
    }
}

#[derive(Resource)]
struct ToastStorage {
    toasts: Toasts,
}

impl Default for ToastStorage {
    fn default() -> Self {
        Self {
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_TOP, (-10.0, 10.0))
                .direction(egui::Direction::TopDown),
        }
    }
}

#[derive(Event)]
pub struct ToastMessage {
    text: String,
    kind: ToastKind,
}

impl ToastMessage {
    pub fn new(text: &str, kind: ToastKind) -> Self {
        Self {
            text: text.to_string(),
            kind,
        }
    }
}

impl From<&ToastMessage> for Toast {
    fn from(value: &ToastMessage) -> Self {
        let duration = match &value.kind {
            ToastKind::Warning => 6.,
            ToastKind::Error => 10.,
            _ => 4.,
        };
        Self {
            text: value.text.clone().into(),
            kind: value.kind,
            options: ToastOptions::default()
                .show_icon(true)
                .duration_in_seconds(duration)
                .show_progress(false),
        }
    }
}

fn read_toast(mut events: EventReader<ToastMessage>, mut storage: ResMut<ToastStorage>) {
    for event in events.read() {
        storage.toasts.add(event.into());
    }

    events.clear();
}

fn show_toast(
    mut storage: ResMut<ToastStorage>,
    mut ctxs: Query<&mut EguiContext, With<PrimaryWindow>>,
) {
    let mut ctx = ctxs.single_mut();
    storage.toasts.show(ctx.get_mut());
}
