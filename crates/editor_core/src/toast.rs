use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui_next::EguiContext;
use egui_dock::egui::{self, Align2};

pub use egui_toast::*;
pub struct ToastUiPlugin;

impl Plugin for ToastUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ToastBasePlugin)
            .add_systems(PostUpdate, show_toast);
    }
}

struct ToastBasePlugin;

impl Plugin for ToastBasePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastStorage>()
            .add_event::<ToastMessage>()
            .add_event::<ClearToastMessage>()
            .add_systems(Update, read_toast)
            .add_systems(PostUpdate, clear_toasts);
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ToastsPerKind {
    pub warning: Vec<String>,
    pub error: Vec<String>,
}

#[derive(Resource)]
pub struct ToastStorage {
    toasts: Toasts,
    pub toasts_per_kind: ToastsPerKind,
}

impl ToastStorage {
    fn add(&mut self, message: &ToastMessage) {
        match &message.kind {
            ToastKind::Warning => self.toasts_per_kind.warning.push(message.text.clone()),
            ToastKind::Error => self.toasts_per_kind.error.push(message.text.clone()),
            _ => (),
        }
    }

    pub fn has_toasts(&self) -> bool {
        !self.toasts_per_kind.warning.is_empty() || !self.toasts_per_kind.error.is_empty()
    }
}

impl Default for ToastStorage {
    fn default() -> Self {
        Self {
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_TOP, (-10.0, 10.0))
                .direction(egui::Direction::TopDown),
            toasts_per_kind: ToastsPerKind::default(),
        }
    }
}

#[derive(Event)]
pub struct ToastMessage {
    text: String,
    kind: ToastKind,
}

#[derive(Event)]
pub struct ClearToastMessage {
    index: usize,
    kind: ToastKind,
    all: bool,
}

impl ClearToastMessage {
    pub const fn error(index: usize) -> Self {
        Self {
            index,
            kind: ToastKind::Error,
            all: false,
        }
    }

    pub const fn warn(index: usize) -> Self {
        Self {
            index,
            kind: ToastKind::Warning,
            all: false,
        }
    }

    pub const fn all() -> Self {
        Self {
            index: 0,
            kind: ToastKind::Error,
            all: true,
        }
    }
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
        let duration = toast_kind_to_duration(value);
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

const fn toast_kind_to_duration(value: &ToastMessage) -> f64 {
    match &value.kind {
        ToastKind::Warning => 6.,
        ToastKind::Error => 10.,
        _ => 4.,
    }
}

fn read_toast(mut events: EventReader<ToastMessage>, mut storage: ResMut<ToastStorage>) {
    for event in events.read() {
        storage.add(event);
        storage.toasts.add(event.into());
    }

    events.clear();
}

fn clear_toasts(mut events: EventReader<ClearToastMessage>, mut storage: ResMut<ToastStorage>) {
    for event in events.read() {
        if event.all {
            storage.toasts_per_kind = ToastsPerKind::default();
        } else {
            match event.kind {
                ToastKind::Warning => {
                    if event.index < storage.toasts_per_kind.warning.len() {
                        storage.toasts_per_kind.warning.remove(event.index);
                    }
                }
                ToastKind::Error => {
                    if event.index < storage.toasts_per_kind.error.len() {
                        storage.toasts_per_kind.error.remove(event.index);
                    }
                }
                _ => {}
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toast_message() {
        let message = ToastMessage::new("Test message", ToastKind::Info);
        assert_eq!(message.text, "Test message");
        assert_eq!(message.kind, ToastKind::Info);
    }

    #[test]
    fn from_toast_message() {
        let message = ToastMessage::new("Test message", ToastKind::Info);
        let toast = Toast::from(&message);
        assert_eq!(toast.text.text(), message.text);
        assert_eq!(toast.kind, message.kind);
        assert_eq!(toast.options.show_icon, true);
        assert_eq!(toast.options.show_progress, false);
        assert_eq!(toast.options.progress(), 1.);
    }

    #[test]
    fn toast_kinds_to_durations() {
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Info)),
            4.
        );
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Warning)),
            6.
        );
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Error)),
            10.
        );
    }

    #[test]
    fn toast_plugin_received_event() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        let storage: &ToastStorage = app.world.get_resource::<ToastStorage>().unwrap();
        assert_eq!(storage.toasts_per_kind.error.len(), 3);
        assert_eq!(storage.toasts_per_kind.warning.len(), 3);
        assert!(storage.has_toasts());
    }

    #[test]
    fn clear_toasts() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        app.world.send_event(ClearToastMessage::error(1));
        app.world.send_event(ClearToastMessage::warn(1));
        app.update();

        let storage: &ToastStorage = app.world.get_resource::<ToastStorage>().unwrap();
        assert_eq!(storage.toasts_per_kind.error.len(), 2);
        assert_eq!(storage.toasts_per_kind.warning.len(), 2);
    }

    #[test]
    fn clear_all_toasts() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        app.world.send_event(ClearToastMessage::all());
        app.update();

        let storage: &ToastStorage = app.world.get_resource::<ToastStorage>().unwrap();
        assert!(!storage.has_toasts());
    }
}
