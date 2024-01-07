use crate::prelude::EditorRegistryBevyExt;
use bevy::{input::gamepad::*, prelude::*, reflect::GetTypeRegistration};
use std::any::TypeId;

pub(crate) trait BevyEvent:
    Event + Default + Resource + Reflect + Send + Clone + 'static + GetTypeRegistration
{
    type T: Event + Clone;

    fn inner_event(&self) -> &Self::T;

    fn inner_path() -> String {
        std::any::type_name::<Self::T>().to_string()
    }

    fn inner_type_id() -> TypeId {
        TypeId::of::<Self::T>()
    }
}

#[derive(Event, Resource, Reflect, Clone)]
#[reflect(Default)]
pub(crate) struct WrappedGamepadEvent(GamepadEvent);

impl Default for WrappedGamepadEvent {
    fn default() -> Self {
        Self(GamepadEvent::Connection(GamepadConnectionEvent {
            gamepad: Gamepad::new(0),
            connection: GamepadConnection::Disconnected,
        }))
    }
}

impl BevyEvent for WrappedGamepadEvent {
    type T = GamepadEvent;

    fn inner_event(&self) -> &Self::T {
        &self.0
    }
}

pub(crate) fn register_events(app: &mut App) -> &mut App {
    app.editor_registry_bevy_event::<WrappedGamepadEvent>()
        .register_type::<WrappedGamepadEvent>()
        .register_type::<GamepadEvent>()
        .register_type::<GamepadConnection>()
        .register_type::<GamepadConnectionEvent>()
        .register_type::<GamepadButtonChangedEvent>()
        .register_type::<GamepadAxisChangedEvent>()
        .register_type::<GamepadInfo>()

    // TODO: add all events and relevant types (https://github.com/rewin123/space_editor/issues/159)
}
