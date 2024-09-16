use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui_dock::egui::{self, Align2};
use space_shared::toast::ToastMessage;

pub use egui_toast::*;
pub struct ToastUiPlugin;

impl Plugin for ToastUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ToastBasePlugin)
            .add_systems(Update, show_toast);
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

fn show_toast(mut storage: ResMut<ToastStorage>, mut ctxs: EguiContexts) {
    storage.toasts.show(ctxs.ctx_mut());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_plugin_received_event() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world()
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        let storage: &ToastStorage = app.world().get_resource::<ToastStorage>().unwrap();
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
            .world()
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        app.world_mut().send_event(ClearToastMessage::error(1));
        app.world_mut().send_event(ClearToastMessage::warn(1));
        app.update();

        let storage: &ToastStorage = app.world().get_resource::<ToastStorage>().unwrap();
        assert_eq!(storage.toasts_per_kind.error.len(), 2);
        assert_eq!(storage.toasts_per_kind.warning.len(), 2);
    }

    #[test]
    fn clear_all_toasts() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world()
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Error));
        app.update();
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Warning));
        app.update();

        app.world_mut().send_event(ClearToastMessage::all());
        app.update();

        let storage: &ToastStorage = app.world().get_resource::<ToastStorage>().unwrap();
        assert!(!storage.has_toasts());
    }

    #[test]
    fn empty_storage_for_info_toasts() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ToastBasePlugin));

        app.update();
        assert!(!app
            .world()
            .get_resource::<ToastStorage>()
            .unwrap()
            .has_toasts());
        app.world_mut()
            .send_event(ToastMessage::new("Test message", ToastKind::Info));
        app.update();

        let storage: &ToastStorage = app.world().get_resource::<ToastStorage>().unwrap();
        assert!(!storage.has_toasts());
    }
}
